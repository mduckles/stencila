use crate::{
    cli::display,
    graphs::{Relation, Resource},
    methods::compile,
    utils::uuids,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use derive_more::{Deref, DerefMut};
use enum_dispatch::enum_dispatch;
use eyre::{eyre, Result};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::{hash_map::Entry, BTreeMap, HashMap};
use stencila_schema::Node;
use strum::Display;
use validator::Contains;

type KernelId = String;

/// The status of a kernel
#[derive(Debug, Clone, JsonSchema, Serialize, Display)]
#[allow(dead_code)]
pub enum KernelStatus {
    Pending,
    Starting,
    Idle,
    Busy,
    Unresponsive,
    Stopping,
    Finished,
    Failed,
}

#[async_trait]
#[enum_dispatch]
pub trait KernelTrait {
    /// Get the name of the kernel's programming language, and/or
    /// check that it is able to execute a given language.
    ///
    /// If a `language` identifier is supplied, e.g. `Some("py")`, and the kernel
    /// can execute that language, should return the kernel's canonical name of the language
    /// e.g. `Ok("python3")`. If the language can not execute the language should
    /// return a `IncompatibleLanguage` error.
    fn language(&self, language: Option<String>) -> Result<String>;

    /// Start the kernel
    async fn start(&mut self) -> Result<()> {
        Ok(())
    }

    /// Stop the kernel
    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    /// Get the status of the kernel
    async fn status(&self) -> KernelStatus {
        KernelStatus::Idle
    }

    /// Get a symbol from the kernel
    async fn get(&mut self, name: &str) -> Result<Node>;

    /// Set a symbol in the kernel
    async fn set(&mut self, name: &str, value: Node) -> Result<()>;

    /// Execute some code in the kernel
    async fn exec(&mut self, code: &str) -> Result<Vec<Node>>;
}

mod default;
use default::*;

#[cfg(feature = "kernels-calc")]
mod calc;

#[cfg(feature = "kernels-jupyter")]
mod jupyter;

#[allow(clippy::large_enum_variant)]
#[enum_dispatch(KernelTrait)]
#[derive(Debug, Clone, JsonSchema, Serialize)]
#[serde(tag = "type")]
pub enum Kernel {
    Default(DefaultKernel),

    #[cfg(feature = "kernels-calc")]
    Calc(calc::CalcKernel),

    #[cfg(feature = "kernels-jupyter")]
    Jupyter(jupyter::JupyterKernel),
}

impl Kernel {
    /// Get a list of available kernels
    #[allow(clippy::vec_init_then_push)]
    pub async fn available() -> Result<Vec<String>> {
        let mut list: Vec<String> = Vec::new();

        #[cfg(feature = "kernels-calc")]
        list.push("calc".to_string());

        #[cfg(feature = "kernels-jupyter")]
        list.append(&mut jupyter::JupyterKernel::available().await?);

        Ok(list)
    }
}

#[derive(Debug, Clone, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct KernelInfo {
    /// The id of the kernel.
    id: String,

    /// The language of the kernel
    language: String,

    /// The status of the kernel
    status: String,
}

#[derive(Debug, Clone, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct SymbolInfo {
    /// The type of the object that the symbol refers to (e.g `Number`, `Function`)
    ///
    /// Should be used as a hint only, to the underlying, native type of the symbol.
    kind: String,

    /// The home kernel of the symbol
    ///
    /// The home kernel of a symbol is the kernel that it was last assigned in.
    /// As such, a symbol's home kernel can change, although this is discouraged.
    home: KernelId,

    /// The time that the symbol was last assigned in the home kernel
    ///
    /// A symbol is considered assigned when  a `CodeChunk` with an `Assign` relation
    /// to the symbol is executed or the `kernel.set` method is called.
    assigned: DateTime<Utc>,

    /// The time that the symbol was last mirrored to other kernels
    ///
    /// A timestamp is recorded for each time that a symbol is mirrored to another
    /// kernel. This allows unnecessary mirroring to be avoided if the symbol has
    /// not been assigned since it was last mirrored to that kernel.
    mirrored: HashMap<KernelId, DateTime<Utc>>,
}

impl SymbolInfo {
    pub fn new(kind: &str, kernel_id: &str) -> Self {
        SymbolInfo {
            kind: kind.into(),
            home: kernel_id.into(),
            assigned: Utc::now(),
            mirrored: HashMap::new(),
        }
    }
}

/// A map of [`KernelId`] to [`Kernel`]
///
/// A `newtype` that exists solely to provide a `Result` (rather than `<Option>`
/// when getting a kernel by id.
#[derive(Debug, Clone, Default, Deref, DerefMut, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
struct KernelMap(BTreeMap<KernelId, Kernel>);

impl KernelMap {
    /// Get a reference to a kernel
    fn get(&self, kernel_id: &str) -> Result<&Kernel> {
        (**self)
            .get(kernel_id)
            .ok_or_else(|| eyre!("Unknown kernel `{}`", kernel_id))
    }

    /// Get a mutable reference to a kernel
    fn get_mut(&mut self, kernel_id: &str) -> Result<&mut Kernel> {
        (**self)
            .get_mut(kernel_id)
            .ok_or_else(|| eyre!("Unknown kernel `{}`", kernel_id))
    }
}

#[derive(Debug, Clone, Default, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct KernelSpace {
    /// The kernels in the document kernel space
    kernels: KernelMap,

    /// The symbols in the document kernel space
    symbols: HashMap<String, SymbolInfo>,
}

impl KernelSpace {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a list of kernels in the kernel space
    pub async fn kernels(&self) -> Vec<KernelInfo> {
        let mut info = Vec::new();
        for (id, kernel) in self.kernels.iter() {
            info.push(KernelInfo {
                id: id.to_string(),
                language: kernel.language(None).unwrap_or_else(|_| "-".to_string()),
                status: kernel.status().await.to_string(),
            })
        }
        info
    }

    /// Get a list of symbols in the kernel space
    ///
    /// Mainly for inspection, in the future may return a list with
    /// more information e.g. the type of symbol.
    pub fn symbols(&self) -> HashMap<String, SymbolInfo> {
        self.symbols.clone()
    }

    /// Get a symbol from the kernel space
    pub async fn get(&mut self, name: &str) -> Result<Node> {
        let symbol_info = self
            .symbols
            .get(name)
            .ok_or_else(|| eyre!("Unknown symbol `{}`", name))?;

        let kernel = self.kernels.get_mut(&symbol_info.home)?;
        kernel.get(name).await
    }

    /// Set a symbol in the kernel space
    pub async fn set(&mut self, name: &str, value: Node, language: &str) -> Result<()> {
        let kernel_id = self.ensure(language).await?;
        tracing::debug!("Setting symbol `{}` in kernel `{}`", name, kernel_id);

        let kernel = self.kernels.get_mut(&kernel_id)?;
        kernel.set(name, value).await?;

        match self.symbols.entry(name.to_string()) {
            Entry::Occupied(mut occupied) => {
                let info = occupied.get_mut();
                info.home = kernel_id;
                info.assigned = Utc::now();
            }
            Entry::Vacant(vacant) => {
                vacant.insert(SymbolInfo::new("", &kernel_id));
            }
        }

        Ok(())
    }

    /// Execute some code in the kernel space
    ///
    /// Symbols that the code uses, but have a different home kernel, are mirrored to the kernel.
    pub async fn exec(
        &mut self,
        code: &str,
        language: &str,
        relations: Option<Vec<(Relation, Resource)>>,
    ) -> Result<Vec<Node>> {
        // Determine the kernel to execute in
        let kernel_id = self.ensure(language).await?;
        tracing::debug!("Executing code in kernel `{}`", kernel_id);

        // Mirror used symbols into the kernel
        if let Some(relations) = &relations {
            for relation in relations {
                let name = if let (Relation::Use(..), Resource::Symbol(symbol)) = relation {
                    if self.symbols.has_element(&symbol.name) {
                        &symbol.name
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                let SymbolInfo {
                    home,
                    assigned,
                    mirrored,
                    ..
                } = self
                    .symbols
                    .get_mut(name)
                    .ok_or_else(|| eyre!("Unknown symbol `{}`", name))?;

                // Skip if home is the target kernel
                if *home == kernel_id {
                    continue;
                }

                // Skip if already mirrored since last assigned
                if let Some(mirrored) = mirrored.get(&kernel_id) {
                    if mirrored >= assigned {
                        continue;
                    }
                }

                tracing::debug!(
                    "Mirroring symbol `{}` from kernel `{}` to kernel `{}`",
                    name,
                    home,
                    kernel_id
                );

                let home_kernel = self.kernels.get_mut(home)?;
                let value = home_kernel.get(name).await?;

                let mirror_kernel = self.kernels.get_mut(&kernel_id)?;
                mirror_kernel.set(name, value).await?;

                match mirrored.entry(kernel_id.clone()) {
                    Entry::Occupied(mut occupied) => {
                        let datetime = occupied.get_mut();
                        *datetime = Utc::now();
                    }
                    Entry::Vacant(vacant) => {
                        vacant.insert(Utc::now());
                    }
                }
            }
        }

        // Execute the code
        let kernel = self.kernels.get_mut(&kernel_id)?;
        let nodes = kernel.exec(code).await?;

        // Record symbols assigned in kernel
        if let Some(relations) = relations {
            for relation in relations {
                let (name, kind) =
                    if let (Relation::Assign(..), Resource::Symbol(symbol)) = relation {
                        (symbol.name, symbol.kind)
                    } else {
                        continue;
                    };

                match self.symbols.entry(name) {
                    Entry::Occupied(mut occupied) => {
                        let info = occupied.get_mut();
                        info.home = kernel_id.clone();
                        info.assigned = Utc::now();
                    }
                    Entry::Vacant(vacant) => {
                        vacant.insert(SymbolInfo::new(&kind, &kernel_id));
                    }
                }
            }
        }

        Ok(nodes)
    }

    /// Ensure that a kernel exists for a language
    ///
    /// Returns a tuple of the kernel's canonical language name and id.
    async fn ensure(&mut self, language: &str) -> Result<KernelId> {
        // Is there already a kernel capable of executing the language?
        for (kernel_id, kernel) in self.kernels.iter_mut() {
            if kernel.language(Some(language.to_string())).is_ok() {
                return Ok(kernel_id.clone());
            }
        }
        // If unable to set in an existing kernel then start a new kernel
        // for the language.
        self.start(language).await
    }

    /// Start a kernel for a language
    async fn start(&mut self, language: &str) -> Result<KernelId> {
        let kernel_id = uuids::generate(uuids::Family::Kernel);
        let mut kernel = match language {
            "none" | "" => DefaultKernel::create(),

            #[cfg(feature = "kernels-calc")]
            "calc" => calc::CalcKernel::create(),

            #[cfg(feature = "kernels-jupyter")]
            _ => jupyter::JupyterKernel::create(&kernel_id, language).await?,

            #[cfg(not(feature = "kernels-jupyter"))]
            _ => bail!(
                "Unable to create an execution kernel for language `{}` because support for Jupyter kernels is not enabled",
                language
            ),
        };
        kernel.start().await?;
        self.kernels.insert(kernel_id.clone(), kernel);

        Ok(kernel_id)
    }

    /// Connect to a running kernel
    async fn connect(&mut self, id_or_path: &str) -> Result<KernelId> {
        #[cfg(not(feature = "kernels-jupyter"))]
        bail!(
            "Unable to connect to running kernel because support for Jupyter kernels is not enabled",
        );

        let (kernel_id, kernel) = jupyter::JupyterKernel::connect(id_or_path).await?;
        self.kernels.insert(kernel_id.clone(), kernel);

        Ok(kernel_id)
    }

    /// Stop one of the kernels and remove it from the kernel space
    async fn stop(&mut self, id: &str) -> Result<()> {
        self.kernels.get_mut(id)?.stop().await?;
        self.kernels.remove(id);
        Ok(())
    }

    /// A read-evaluate-print function
    ///
    /// Primarily intended for use in interactive mode as an execution REPL.
    /// Adds execution related shortcuts e.g. `%symbols` for changing the language.
    pub async fn repl(&mut self, code: &str, language: &str) -> display::Result {
        if !code.is_empty() {
            let code = code.replace("\\n", "\n");
            if code == "%symbols" {
                let symbols = self.symbols();
                display::value(symbols)
            } else if code == "%kernels" {
                let kernels = self.kernels().await;
                display::value(kernels)
            } else {
                // Compile the code so that we can use the relations to determine variables that
                // are assigned or used (needed for variable mirroring).
                let relations = compile::code::compile("<cli>", &code, language);
                let nodes = self.exec(&code, language, Some(relations)).await?;
                match nodes.len() {
                    0 => display::nothing(),
                    1 => display::value(nodes[0].clone()),
                    _ => display::value(nodes),
                }
            }
        } else {
            display::nothing()
        }
    }
}

#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use crate::cli::display;
    use once_cell::sync::Lazy;
    use serde_json::json;
    use structopt::StructOpt;
    use tokio::sync::Mutex;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Manage kernels",
        setting = structopt::clap::AppSettings::ColoredHelp,
        setting = structopt::clap::AppSettings::VersionlessSubcommands
    )]
    pub struct Command {
        #[structopt(subcommand)]
        pub action: Action,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder
    )]
    pub enum Action {
        Available(Available),
        Running(Running),
        Execute(Execute),
        Start(Start),
        Connect(Connect),
        Stop(Stop),
        Status(Status),
        Show(Show),
    }

    impl Command {
        pub async fn run(self) -> display::Result {
            let Self { action } = self;
            match action {
                Action::Available(action) => action.run().await,
                Action::Running(action) => action.run().await,
                Action::Execute(action) => action.run().await,
                Action::Start(action) => action.run().await,
                Action::Connect(action) => action.run().await,
                Action::Stop(action) => action.run().await,
                Action::Status(action) => action.run().await,
                Action::Show(action) => action.run().await,
            }
        }
    }

    /// List the kernels that are available on this machine
    ///
    /// The list of available kernels includes those that are built into the Stencila
    /// binary (e.g. `calc`) as well as any Jupyter kernels installed on the machine
    /// (e.g. `python`).
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Available {}
    impl Available {
        pub async fn run(&self) -> display::Result {
            let list = Kernel::available().await?;
            display::value(list)
        }
    }

    /// List the kernels (and servers) that are currently running on this machine
    ///
    /// This command scans the Jupyter `runtime` directory to get a list of running
    /// Jupyter notebook servers. It then gets a list of kernels from the REST API
    /// of each of those servers.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Running {}
    impl Running {
        pub async fn run(&self) -> display::Result {
            #[cfg(feature = "kernels-jupyter")]
            {
                let kernels = jupyter::JupyterKernel::running().await?;
                let servers = jupyter::JupyterServer::running().await?;
                display::value(json!({
                    "kernels": kernels,
                    "servers": servers
                }))
            }
            #[cfg(not(feature = "kernels-jupyter"))]
            {
                bail!("Jupyter kernels are not enabled")
            }
        }
    }

    /// A lazily initialized kernel space for the execute command. Required so that
    /// kernel state is maintained in successive calls to `Execute::run` when in
    /// interactive mode
    static KERNEL_SPACE: Lazy<Mutex<KernelSpace>> = Lazy::new(|| Mutex::new(KernelSpace::new()));

    /// Execute code within a temporary "kernel space"
    ///
    /// Mainly intended for testing that Stencila is able to talk
    /// to Jupyter kernels and execute code within them.
    ///
    /// Use the `--lang` option to specify which language the code should be executed
    /// in e.g.,
    ///
    /// > kernels execute Math.PI --lang=javascript
    ///
    /// In interactive mode, you can set the command prefix to "stay" in a particular
    /// language and mimic a REPL in that language e.g.,
    ///
    /// > >> kernels execute --lang=javascript
    /// > let r = 10
    /// > 2 * Math.PI * r
    ///
    /// If a kernel is not yet running for the language then one will be started
    /// (if installed on the machine).
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Execute {
        /// Code to execute within the document's kernel space
        ///
        /// This code will be run after all executable nodes in the document
        /// have been run.
        // Using a Vec and the `multiple` option allows for spaces in the code
        #[structopt(multiple = true)]
        code: Vec<String>,

        /// The programming language of the code
        #[structopt(short, long, default_value = "calc")]
        lang: String,
    }
    impl Execute {
        pub async fn run(&self) -> display::Result {
            KERNEL_SPACE
                .lock()
                .await
                .repl(&self.code.join(" "), &self.lang)
                .await
        }
    }

    /// Start a kernel for a particular programming language
    ///
    /// Mainly intended for testing that a Jupyter kernel can be
    /// started successfully e.g.,
    ///
    /// > kernels start python
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Start {
        /// The programming language of the kernel
        lang: String,
    }
    impl Start {
        pub async fn run(&self) -> display::Result {
            KERNEL_SPACE.lock().await.ensure(&self.lang).await?;
            tracing::info!("Started kernel for language `{}`", self.lang);
            display::nothing()
        }
    }

    /// Stop a kernel
    ///
    /// Mainly intended for testing that Jupyter kernels are successfully
    /// stopped (e.g. cleanup of connection files).
    ///
    /// Only kernels that were started by Stencila can be stopped. A kernel
    /// that were started externally by a Jupyter server and then connected to
    /// will still run but Stencila will clone any connections to it.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Stop {
        /// The id of the kernel (see `kernels status`)
        id: String,
    }
    impl Stop {
        pub async fn run(&self) -> display::Result {
            KERNEL_SPACE.lock().await.stop(&self.id).await?;
            tracing::info!("Stopped kernel `{}`", self.id);
            display::nothing()
        }
    }

    /// Connect to a running kernel
    ///
    /// Mainly intended for testing that Stencila is able to connect
    /// to an existing kernel (e.g. one that was started from Jupyter notebook).
    ///
    /// To get a list of externally started kernels that can be connected to run,
    ///
    /// > kernels running
    ///
    /// and then connect to a kernel using its Jupyter id e.g.,
    ///
    /// > kernels connect beaac32f-32a4-46bc-9940-186a14d9acc9
    ///
    /// Alternatively, use the path (relative or absolute) of the Jupyter notebook
    /// whose (already started) kernel you wish to connect to e.g.,
    ///
    /// > kernels connect ../main.ipynb
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Connect {
        /// The id of the kernel e.g. `31248fc2-38d0-4d11-80a1-f8a1bd3842fb`
        /// or the relative path of the notebook
        id_or_path: String,
    }
    impl Connect {
        pub async fn run(&self) -> display::Result {
            let mut kernels = KERNEL_SPACE.lock().await;
            let id = kernels.connect(&self.id_or_path).await?;
            tracing::info!("Connected to kernel `{}`", id);
            display::nothing()
        }
    }

    /// Get a list of the kernels in the current kernel space
    ///
    /// Mainly intended for interactive mode testing / inspection. Note that
    /// for a kernel to be in this list it must have either been started by Stencila,
    ///
    /// > kernels start r
    ///
    /// or connected to from Stencila,
    ///  
    /// > kernels connect beaac32f-32a4-46bc-9940-186a14d9acc9
    ///
    /// To get a list of externally started kernels that can be connected to run,
    ///
    /// > kernels running
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Status {}
    impl Status {
        pub async fn run(&self) -> display::Result {
            let status = KERNEL_SPACE.lock().await.kernels().await;
            display::value(status)
        }
    }

    /// Show the details of a current kernel
    ///
    /// Mainly intended for interactive mode testing / inspection.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Show {
        /// The id of the kernel (see `kernels status`)
        id: String,
    }
    impl Show {
        pub async fn run(&self) -> display::Result {
            let kernels = KERNEL_SPACE.lock().await;
            let kernel = kernels.kernels.get(&self.id)?;
            display::value(kernel)
        }
    }
}
