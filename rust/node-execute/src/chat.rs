use common::{itertools::Itertools, tokio};
use schema::{
    shortcuts::{cc, mb, p, t},
    Author, Block, Chat, ChatMessage, ChatMessageGroup, ChatMessageOptions, MessageRole,
    SoftwareApplication,
};

use crate::{interrupt_impl, prelude::*};

impl Executable for Chat {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Chat {node_id}");

        // Call `prompt.compile` directly because a `PromptBlock` that
        // is not a `Block::PromptBlock` variant is not walked over
        self.prompt.compile(executor).await;

        // Continue walk to compile other properties
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing Chat {node_id}");

        // Check if this chat is to be executed: node ids contain this chat.
        // This is more restrictive than other nodes types: a chat is only executed
        // explicitly.
        let prepare_self = executor.node_ids.iter().flatten().any(|id| id == &node_id);

        // If not to be executed, then return early and continue walking document
        // to prepare nodes in the chat's `content`
        if !prepare_self {
            return WalkControl::Continue;
        }

        // Set execution status
        self.options.execution_status = Some(ExecutionStatus::Pending);
        executor.patch(
            &node_id,
            [set(NodeProperty::ExecutionStatus, ExecutionStatus::Pending)],
        );

        // Do not continue to prepare nodes in `content` because the
        // chat itself is being executed
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if !matches!(
            self.options.execution_status,
            Some(ExecutionStatus::Pending)
        ) {
            // Chat itself not marked as pending so continue to execute nodes in `content`
            return WalkControl::Continue;
        }

        tracing::debug!("Executing Chat {node_id}");

        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, ExecutionStatus::Running),
                none(NodeProperty::ExecutionMessages),
            ],
        );

        let started = Timestamp::now();

        // If there are no messages yet, and the prompt block contains a query
        // then use it as the first message
        if let (true, Some(query)) = (self.content.is_empty(), &self.prompt.query) {
            let mut parts = Vec::new();

            if let Some(instruction_type) = &self.prompt.instruction_type {
                parts.push(instruction_type.to_string());
            }

            if let Some(relative_position) = &self.prompt.relative_position {
                parts.push(relative_position.to_string().to_lowercase());
            }

            if let Some(node_type) = &self.prompt.node_types.iter().flatten().next() {
                parts.push(node_type.to_string());
            }

            parts.push(query.to_string());

            let text = parts.join(" ");

            let message = Block::ChatMessage(ChatMessage {
                role: MessageRole::User,
                content: vec![p([t(text)])],
                ..Default::default()
            });

            self.content.push(message.clone());
            executor.patch(&node_id, [push(NodeProperty::Content, message)])
        }

        // TODO: construct a model task from all the messages in this chat
        for block in self.content.iter_mut() {
            if let Block::ChatMessage(msg) = block {
                // If the message does not have an execution status then
                // set to succeeded. This is important for user messages because
                // it triggers a re-render of the Web Component.
                if msg.options.execution_status.is_none() {
                    executor.patch(
                        &msg.node_id(),
                        [set(
                            NodeProperty::ExecutionStatus,
                            ExecutionStatus::Succeeded,
                        )],
                    );
                }
            }
        }

        // Add a new model message to the chat, with no content, so the user
        // can see it as running

        let model_ids = self
            .model_parameters
            .model_ids
            .clone()
            .unwrap_or_else(|| vec!["stencila/router".to_string()]);
        let replicates = self.model_parameters.replicates.unwrap_or(1) as usize;

        let model_ids = model_ids
            .into_iter()
            .flat_map(|x| vec![x; replicates])
            .collect_vec();

        let mut messages = model_ids
            .into_iter()
            .map(|model_id| ChatMessage {
                role: MessageRole::Model,
                options: Box::new(ChatMessageOptions {
                    author: Some(Author::SoftwareApplication(SoftwareApplication {
                        id: Some(model_id),
                        name: "Model".into(), // TODO
                        ..Default::default()
                    })),
                    execution_status: Some(ExecutionStatus::Running),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .collect_vec();

        let message_ids = messages
            .iter()
            .map(|message| message.node_id())
            .collect_vec();

        let block = if messages.len() == 1 {
            Block::ChatMessage(messages.swap_remove(0))
        } else {
            Block::ChatMessageGroup(ChatMessageGroup {
                messages,
                ..Default::default()
            })
        };
        executor.patch(&node_id, [push(NodeProperty::Content, block)]);

        // TODO: replace this simulation with actual model generated content
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        let messages = None;
        let contents = message_ids.iter().map(|_| {
            vec![
                p([t("This is placeholder content for model response. Laborum duis ut cillum ex incididunt officia ex aliquip. Here is some executable code:")]),
                cc("plot(1)", Some("r")),
                p([t("Here is some math:")]),
                mb("E = mc ^ 2 * \\pi", Some("tex")),
                p([t("Last paragraph of the model response. Enim pariatur in voluptate reprehenderit Lorem quis esse cupidatat minim. Anim ipsum exercitation eiusmod laboris nostrud ullamco commodo amet nisi sit. Aute sunt quis ad tempor consectetur eiusmod non est. Laborum ea et esse irure nostrud labore irure. Officia labore velit cillum id cupidatat aliquip aute fugiat ea deserunt esse aliqua in. Non amet est eu enim mollit velit fugiat et ullamco cillum. Reprehenderit reprehenderit adipisicing laboris veniam in aute aute aliqua..")]),
            ]
        });

        // Set the status of each message
        for block in self.content.iter_mut() {
            if let Block::ChatMessage(msg) = block {
                if !matches!(
                    msg.options.execution_status,
                    Some(ExecutionStatus::Succeeded)
                ) {
                    executor.patch(
                        &msg.node_id(),
                        [set(
                            NodeProperty::ExecutionStatus,
                            ExecutionStatus::Succeeded,
                        )],
                    );
                }
            }
        }

        let ended = Timestamp::now();

        let status = execution_status(&messages);
        let required = execution_required_status(&status);
        let duration = execution_duration(&started, &ended);
        let count = self.options.execution_count.unwrap_or_default() + 1;

        // Patch the status and content of the model messages
        for (message_id, content) in message_ids.iter().zip(contents) {
            executor.patch(
                message_id,
                [
                    set(NodeProperty::ExecutionStatus, status.clone()),
                    append(NodeProperty::Content, content),
                ],
            );
        }

        // Patch the chat, including appending a new, empty user message
        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, status),
                set(NodeProperty::ExecutionRequired, required),
                set(NodeProperty::ExecutionMessages, messages),
                set(NodeProperty::ExecutionDuration, duration),
                set(NodeProperty::ExecutionEnded, ended),
                set(NodeProperty::ExecutionCount, count),
            ],
        );

        // Break walk because the chat has been updated
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn interrupt(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Interrupting Chat {node_id}");

        interrupt_impl!(self, executor, &node_id);

        // Continue to interrupt executable nodes in `content`
        WalkControl::Continue
    }
}
