// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Indicates the action (create, update or delete) to derive for a `Form`.
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, MarkdownCodec, TextCodec, SmartDefault, Read, Write)]
#[serde(crate = "common::serde")]
pub enum FormDeriveAction {
    #[default]
    Create,
    Update,
    Delete,
    UpdateOrDelete,
}
