use std::sync::Arc;

use assistant::{
    format::Format, Assistant, AssistantIO, AssistantType, GenerateOptions, GenerateOutput,
    GenerateTask,
};
use common::{
    async_trait::async_trait,
    eyre::{bail, Result},
    inflector::Inflector,
    serde::{Deserialize, Serialize},
    tokio::sync::Mutex,
};

use crate::{plugins, Plugin, PluginInstance};

/// A assistant provided by a plugin
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "common::serde")]
pub struct PluginAssistant {
    /// The id of the assistant
    id: String,

    /// The name of the assistant
    ///
    /// Will be extracted from the id if not supplied
    name: Option<String>,

    /// The input types that the assistant supports
    #[serde(default)]
    inputs: Vec<AssistantIO>,

    /// The output types that the assistant supports
    #[serde(default)]
    outputs: Vec<AssistantIO>,

    /// The format that the content of the instruction should
    /// be formatted using for use in the system prompt
    #[serde(alias = "content-format")]
    content_format: Option<Format>,

    /// The system prompt template
    #[serde(alias = "system-prompt")]
    system_prompt: Option<String>,

    /// The plugin that provides this assistant
    ///
    /// Used to be able to create a plugin instance, which in
    /// turn is used to create a assistant instance.
    #[serde(skip)]
    plugin: Option<Plugin>,

    /// The plugin instance for this assistant. Used to avoid starting
    /// a new instance for each call to the assistant.
    ///
    /// This needs to be a `Arc<Mutex>` because the `perform_task` method async
    /// but is not `&mut self`. So, this is needed for "interior mutability" across
    /// calls to that method.
    #[serde(skip)]
    plugin_instance: Arc<Mutex<Option<PluginInstance>>>,
}

impl PluginAssistant {
    /// Bind a plugin to this assistant so that it can be started (by starting the plugin first)
    pub fn bind(&mut self, plugin: &Plugin) {
        self.plugin = Some(plugin.clone());
    }
}

#[async_trait]
impl Assistant for PluginAssistant {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> String {
        self.name.clone().unwrap_or_else(|| {
            let id = self.id.clone();
            let name = id
                .rsplit_once('/')
                .map(|(.., name)| name.split_once('-').map_or(name, |(name, ..)| name))
                .unwrap_or(&id);
            name.to_title_case()
        })
    }

    fn version(&self) -> String {
        self.plugin
            .as_ref()
            .map(|plugin| plugin.version.to_string())
            .unwrap_or_default()
    }

    fn r#type(&self) -> AssistantType {
        match &self.plugin {
            Some(plugin) => {
                let mut name = plugin.name.clone();
                if plugin.linked {
                    name += " (linked)";
                }
                AssistantType::Plugin(name)
            }
            None => AssistantType::Plugin("unknown".to_string()),
        }
    }

    fn supported_inputs(&self) -> &[AssistantIO] {
        if self.inputs.is_empty() {
            &[AssistantIO::Text]
        } else {
            &self.inputs
        }
    }

    fn supported_outputs(&self) -> &[AssistantIO] {
        if self.outputs.is_empty() {
            &[AssistantIO::Text]
        } else {
            &self.outputs
        }
    }

    async fn perform_task(
        &self,
        task: &GenerateTask,
        options: &GenerateOptions,
    ) -> Result<GenerateOutput> {
        // Create the plugin instance if necessary
        let mut guard = self.plugin_instance.lock().await;
        let instance = match &mut *guard {
            Some(instance) => instance,
            None => {
                let Some(plugin) = self.plugin.as_ref() else {
                    bail!("Not bound yet")
                };

                let inst = plugin.start(None).await?;
                *guard = Some(inst);
                guard.as_mut().unwrap()
            }
        };

        //  Prepare the task
        let mut task = task.clone();
        if self.content_format.is_some() || self.system_prompt.is_some() {
            task.prepare(
                Some(self),
                self.content_format.as_ref(),
                self.system_prompt.as_ref(),
            )
            .await?;
        }

        // Call the plugin method
        #[derive(Serialize)]
        #[serde(crate = "common::serde")]
        struct Params {
            assistant: String,
            task: GenerateTask,
            options: GenerateOptions,
        }
        let output: GenerateOutput = instance
            .call(
                "assistant_execute",
                Params {
                    assistant: self.id(),
                    task: task.clone(),
                    options: options.clone(),
                },
            )
            .await?;

        // Post process the output
        let format = if output.format == Format::Unknown {
            task.format().clone()
        } else {
            output.format.clone()
        };
        GenerateOutput::from_plugin(output, self, &format, task.instruction(), options).await
    }
}

/// List all the assistants provided by plugins
pub async fn list() -> Result<Vec<Arc<dyn Assistant>>> {
    Ok(plugins()
        .await
        .into_iter()
        .flat_map(|plugin| plugin.assistants())
        .collect())
}
