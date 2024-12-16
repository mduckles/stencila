use common::tokio;
use schema::{
    shortcuts::{cc, mb, p, t},
    Block, Chat, ChatMessage, MessageRole,
};

use crate::{interrupt_impl, prelude::*};

impl Executable for Chat {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, _executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Chat {node_id}");

        // Continue walk to compile nodes in `content` and `title`
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Preparing Chat {node_id}");

        // Check if this chat is to be executed: no node ids specified
        // or node ids contain this chat or any chat messages.
        let prepare_self = executor.node_ids.is_none()
            || executor
                .node_ids
                .iter()
                .flatten()
                .any(|node_id| matches!(node_id.nick(), "cht" | "chm"));

        // If not, then return early and continue walking document to prepare
        // nodes in `content`
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
            // Continue to execute nodes in `content`
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

        // TODO: if the chat is associated with a document then
        // execute within the associated document's kernels.

        // TODO: construct a model task from all the messages in this chat
        for block in self.content.iter_mut() {
            if let Block::ChatMessage(msg) = block {
                if msg.options.execution_status.is_none() {
                    executor.patch(
                        &msg.node_id(),
                        [
                            set(NodeProperty::ExecutionStatus, ExecutionStatus::Running),
                            none(NodeProperty::ExecutionMessages),
                        ],
                    );
                }
            }
        }

        // TODO: replace this simulation with actual model generated content
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        let messages = None;
        let content = vec![
            p([t("This is placeholder content for model response. Laborum duis ut cillum ex incididunt officia ex aliquip. Here is some executable code:")]),
            cc("plot(1)", Some("r")),
            p([t("Here is some math:")]),
            mb("E = mc ^ 2 * \\pi", Some("tex")),
            p([t("Last paragraph of the model response. Enim pariatur in voluptate reprehenderit Lorem quis esse cupidatat minim. Anim ipsum exercitation eiusmod laboris nostrud ullamco commodo amet nisi sit. Aute sunt quis ad tempor consectetur eiusmod non est. Laborum ea et esse irure nostrud labore irure. Officia labore velit cillum id cupidatat aliquip aute fugiat ea deserunt esse aliqua in. Non amet est eu enim mollit velit fugiat et ullamco cillum. Reprehenderit reprehenderit adipisicing laboris veniam in aute aute aliqua..")]),
        ];
        let chat_messages = vec![model_chat_message(content), user_chat_message()];

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

        executor.patch(
            &node_id,
            [
                append(NodeProperty::Content, chat_messages),
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

/// Create an empty user chat message
fn user_chat_message() -> Block {
    Block::ChatMessage(ChatMessage {
        role: MessageRole::User,
        ..Default::default()
    })
}

/// Create a model chat message
fn model_chat_message(content: Vec<Block>) -> Block {
    Block::ChatMessage(ChatMessage {
        role: MessageRole::Model,
        content,
        ..Default::default()
    })
}
