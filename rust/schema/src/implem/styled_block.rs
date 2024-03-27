use codec_losses::lost_options;

use crate::{prelude::*, StyledBlock};

impl MarkdownCodec for StyledBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        let fence = ":".repeat(3 + context.depth * 2);

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, style_language))
            .merge_losses(lost_options!(
                self.options,
                compilation_digest,
                compilation_messages,
                css,
                class_list
            ))
            .push_str(&fence)
            .push_str(" {")
            .push_prop_str("code", &self.code)
            .push_str("}\n\n")
            .increase_depth()
            .push_prop_fn("content", |context| self.content.to_markdown(context))
            .decrease_depth()
            .push_str(&fence)
            .newline()
            .exit_node()
            .newline();
    }
}
