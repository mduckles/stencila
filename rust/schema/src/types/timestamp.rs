// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::integer::Integer;
use super::string::String;
use super::time_unit::TimeUnit;

/// A value that represents a point in time.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "Timestamp")]
#[jats(elem = "timestamp", special)]
pub struct Timestamp {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Timestamp"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The time, in `timeUnit`s, before or after the Unix Epoch (1970-01-01T00:00:00Z).
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub value: Integer,

    /// The time unit that the `value` represents.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub time_unit: TimeUnit,
}

impl Timestamp {
    pub fn new(value: Integer, time_unit: TimeUnit) -> Self {
        Self {
            value,
            time_unit,
            ..Default::default()
        }
    }
}
