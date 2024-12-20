use common::serde_yaml;
use node_strip::{StripNode, StripTargets};

use crate::{prelude::*, Chat};

impl Chat {
    /// Custom implementation of [`PatchNode::apply`].
    ///
    /// Only allow operations on the `content` vector if the chat is not nested.
    pub fn apply_with(
        &mut self,
        path: &mut PatchPath,
        op: &PatchOp,
        context: &mut PatchContext,
    ) -> Result<bool> {
        if let Some(PatchSlot::Property(NodeProperty::Content)) = path.front() {
            // Only apply patches to the content of the chat if the patch is
            // associated with no, or a lossless, format, or if it is a root
            // node (not nested)
            let lossless_format = context
                .format
                .as_ref()
                .map(|format| format.is_lossless())
                .unwrap_or(true);
            let is_root = self.is_ephemeral.is_none();

            if lossless_format || is_root {
                path.pop_front();
                self.content.apply(path, op.clone(), context)?;
            }

            return Ok(true);
        }

        Ok(false)
    }
}

impl MarkdownCodec for Chat {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        // If not the root node (i.e. within an `Article` or other document) then
        // just represent as a single line so that the user knows that the chat is there
        // and can interact with it (e.g. via code lenses or key bindings)
        if !context.is_root() {
            context
                .enter_node(self.node_type(), self.node_id())
                .push_str("::: chat\n\n")
                .exit_node();
            return;
        }

        // The following is based on `Article::to_markdown` but with some differences
        // (e.g. not yet supporting authors)

        context.enter_node(self.node_type(), self.node_id());

        // Create a header version of self that has no content and can be stripped
        let mut header = Self {
            // Avoid serializing content unnecessarily
            content: Vec::new(),
            ..self.clone()
        };

        // Strip properties from header that are designated as not supported by Markdown.
        // This would be better to do based on the "patch formats" declaration in the
        // schema but that is not accessible from here. So we have to do it "manually"
        header.strip(&StripTargets {
            scopes: vec![
                StripScope::Provenance,
                StripScope::Execution,
                StripScope::Code,
                StripScope::Output,
                StripScope::Archive,
            ],
            ..Default::default()
        });
        header.options.authors = None;

        let mut yaml = serde_yaml::to_value(header).unwrap_or_default();
        if let Some(yaml) = yaml.as_mapping_mut() {
            // Remove the (now empty) content array
            yaml.remove("content");

            // Encode YAML header
            let yaml = serde_yaml::to_string(&yaml).unwrap_or_default();
            context.push_str("---\n");
            context.push_str(&yaml);
            context.push_str("---\n\n");
        }

        context.push_prop_fn(NodeProperty::Content, |context| {
            self.content.to_markdown(context)
        });

        context.append_footnotes();

        context.exit_node_final();
    }
}
