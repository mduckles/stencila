use codec_info::lost_options;

use crate::{prelude::*, Link};

impl DomCodec for Link {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .enter_elem_attrs("a", [("href", &self.target)]);

        if let Some(title) = &self.title {
            context.push_attr("title", title);
        }

        if let Some(rel) = &self.rel {
            context.push_attr("rel", rel);
        }

        if let Some(id) = &self.id {
            context.push_attr("@id", id);
        }

        context
            .push_slot_fn("span", "content", |context| self.content.to_dom(context))
            .exit_elem()
            .exit_node();
    }
}

impl MarkdownCodec for Link {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, rel))
            .push_str("[")
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .push_str("](")
            .push_prop_str(NodeProperty::Target, &self.target);

        if let Some(title) = &self.title {
            context
                .push_str(" \"")
                .push_prop_fn(NodeProperty::Title, |context| title.to_markdown(context))
                .push_str("\"");
        }

        context.push_str(")").exit_node();
    }
}
