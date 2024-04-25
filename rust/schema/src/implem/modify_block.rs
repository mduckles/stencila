use codec_info::lost_options;

use crate::{prelude::*, ModifyBlock, ModifyOperation, SuggestionStatus};

impl MarkdownCodec for ModifyBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .push_semis()
            .push_str(" modify");

        if let Some(status @ (SuggestionStatus::Accepted | SuggestionStatus::Rejected)) =
            &self.suggestion_status
        {
            context.push_str(" ").push_prop_str(
                NodeProperty::SuggestionStatus,
                &status.to_string().to_lowercase(),
            );
        }

        if self.content.is_empty() {
            context.newline();
        } else {
            context
                .push_str("\n\n")
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                });
        }

        context.push_semis().push_str(" with\n\n");

        let modified =
            ModifyOperation::apply_many(&self.operations, &self.content).unwrap_or_default();
        context
            .push_prop_fn(NodeProperty::Operations, |context| {
                modified.to_markdown(context)
            })
            .push_semis()
            .newline()
            .exit_node()
            .newline();
    }
}
