use codec_losses::lost_options;

use crate::{prelude::*, Division};

impl Division {
    pub fn to_markdown_special(&self, context: &MarkdownEncodeContext) -> (String, Losses) {
        let mut losses = lost_options!(
            self,
            id,
            compilation_digest,
            compilation_errors,
            css,
            classes
        );
        
        let fence = ":".repeat(3 + context.depth * 2);

        let lang = self
            .style_language
            .as_ref()
            .map(|lang| format!(" {lang}"))
            .unwrap_or_default();

        let (md, md_losses) = self.content.to_markdown(&MarkdownEncodeContext {
            depth: context.depth + 1,
        });
        losses.merge(md_losses);

        let md = [
            &fence, &lang, " {", &self.code, "}", "\n\n", &md, &fence, "\n\n",
        ]
        .concat();

        (md, losses)
    }
}
