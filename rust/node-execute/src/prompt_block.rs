use std::{
    hash::{Hash, Hasher},
    ops::Deref,
    path::Path,
    sync::Arc,
};

use common::{
    eyre::{OptionExt, Result},
    seahash::SeaHasher,
    tokio::sync::RwLock,
};
use kernels::Kernels;
use prompts::prompt::{KernelsContext, PromptContext};
use schema::{replicate, CompilationDigest, PromptBlock};

use crate::prelude::*;

impl Executable for PromptBlock {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling PromptBlock {node_id}");

        let mut hash = SeaHasher::new();
        if let Some(value) = &self.instruction_type {
            value.to_string().hash(&mut hash);
        }
        if let Some(value) = &self.hint {
            value.hash(&mut hash);
        }
        if let Some(value) = &self.target {
            value.hash(&mut hash);
        }
        let state_digest = hash.finish();

        let compilation_digest = CompilationDigest::new(state_digest);

        if self.target.is_none()
            || (self
                .target
                .as_ref()
                .map(|target| target.ends_with("?"))
                .unwrap_or_default()
                && Some(&compilation_digest) != self.options.compilation_digest.as_ref())
        {
            if let Some(prompt) =
                prompts::infer(&self.instruction_type, &self.hint.as_deref()).await
            {
                let id = prompt
                    .id
                    .as_ref()
                    .map(|id| prompts::shorten(id, &self.instruction_type))
                    .map(|id| [&id, "?"].concat());

                self.target = id.clone();
                executor.patch(&node_id, [set(NodeProperty::Target, id)]);
            }
        }

        let execution_required =
            execution_required_digests(&self.options.execution_digest, &compilation_digest);

        self.options.compilation_digest = Some(compilation_digest.clone());
        self.options.execution_required = Some(execution_required.clone());
        executor.patch(
            &node_id,
            [
                set(NodeProperty::CompilationDigest, compilation_digest),
                set(NodeProperty::ExecutionRequired, execution_required),
            ],
        );

        // Break walk because `content` is compiled in `execute`
        // and do not want headings, figures etc to be compiled
        // in main document
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing PromptBlock {node_id}");

        // Set execution status
        if let Some(status) = executor.node_execution_status(
            self.node_type(),
            &node_id,
            &self.execution_mode,
            &self.options.compilation_digest,
            &self.options.execution_digest,
        ) {
            self.options.execution_status = Some(status.clone());
            executor.patch(&node_id, [set(NodeProperty::ExecutionStatus, status)]);
        }

        // Break walk because `content` is prepared in `execute` and
        // do not want headings, paragraphs etc to be added to context
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Executing PromptBlock {node_id}");

        // Always execute, so mark as running
        let status = ExecutionStatus::Running;
        self.options.execution_status = Some(status.clone());
        executor.patch(&node_id, [set(NodeProperty::ExecutionStatus, status)]);

        // Get the prompt
        // TODO: separate error if target is None
        let target = self.target.clone().unwrap_or_default();
        let instruction_type = self.instruction_type.clone().unwrap_or_default();
        let prompt = match prompts::get(&target, &instruction_type).await {
            Ok(prompt) => prompt,
            Err(error) => {
                executor.patch(
                    &node_id,
                    [
                        set(NodeProperty::ExecutionStatus, ExecutionStatus::Exceptions),
                        set(
                            NodeProperty::ExecutionMessages,
                            vec![error_to_execution_message("While getting prompt", error)],
                        ),
                    ],
                );
                return WalkControl::Break;
            }
        };

        let started = Timestamp::now();
        let mut messages = Vec::new();

        // Replicate the content of the prompt so that the prompt block has different ids.
        // Given that the same prompt could be used multiple times in a doc, if we don't
        // do this there could be clashes.
        let content = replicate(&prompt.content).unwrap_or_default();

        // Set content here and via patch
        self.content = Some(content.clone());
        executor.patch(
            &node_id,
            [
                // It is important to use `none` and `append` here because
                // the latter retains node ids so they are the same as in `self.content`
                // TODO: consider doing a merge rather than full replacement. Replacement
                // seems to cause large, slow diffs in DOMs (do to a whole lot of new ids?)
                none(NodeProperty::Content),
                append(NodeProperty::Content, content),
            ],
        );

        // Execute content of prompt
        let home = prompt.home();
        match prompt_executor(&home, executor).await {
            Ok(mut prompt_executor) => {
                prompt_executor.directory_stack.push(home);
                if let Err(error) = prompt_executor
                    .compile_prepare_execute(&mut self.content)
                    .await
                {
                    messages.push(error_to_execution_message("While executing prompt", error));
                }
                prompt_executor.directory_stack.pop();
            }
            Err(error) => {
                messages.push(error_to_execution_message(
                    "While creating prompt executor",
                    error,
                ));
            }
        };

        let ended = Timestamp::now();
        let messages = (!messages.is_empty()).then_some(messages);

        let status = execution_status(&messages);
        let required = execution_required_status(&status);
        let duration = execution_duration(&started, &ended);
        let count = self.options.execution_count.unwrap_or_default() + 1;
        let compilation_digest = self.options.compilation_digest.clone();

        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, status),
                set(NodeProperty::ExecutionRequired, required),
                set(NodeProperty::ExecutionMessages, messages),
                set(NodeProperty::ExecutionDuration, duration),
                set(NodeProperty::ExecutionEnded, ended),
                set(NodeProperty::ExecutionCount, count),
                set(NodeProperty::ExecutionDigest, compilation_digest),
            ],
        );

        // Break walk because already walked over content with the new executor
        WalkControl::Break
    }
}

/// Create a new executor to execute a prompt
async fn prompt_executor(home: &Path, executor: &Executor) -> Result<Executor> {
    // Create a prompt context
    // TODO: allow prompts to specify whether they need various parts of context
    // as an optimization, particularly to avoid getting kernel contexts unnecessarily.
    let context = PromptContext {
        instruction: executor.instruction_context.clone(),
        document: Some(executor.document_context.clone()),
        kernels: Some(KernelsContext::from_kernels(executor.kernels.read().await.deref()).await?),
    };

    // Create a new kernel instance for the prompt context
    let kernel = kernels::get("quickjs")
        .await
        .ok_or_eyre("QuickJS kernel not available")?;
    let kernel_instance = context.into_kernel().await?;

    // Create a set of kernels to execute the prompt and add the kernel instance to it
    let mut kernels = Kernels::new(home);
    kernels
        .add_instance(Arc::new(kernel), kernel_instance)
        .await?;

    // Create the new executor using the set of kernels
    let executor = Executor::new(
        home.to_path_buf(),
        Arc::new(RwLock::new(kernels)),
        executor.patch_sender.clone(),
        None,
        None,
    );

    Ok(executor)
}
