use codec::{
    common::{
        async_trait::async_trait,
        eyre::Result,
        serde_json::{self, Map, Value},
    },
    format::Format,
    schema::Node,
    status::Status,
    Codec, DecodeOptions, EncodeOptions, Losses,
};

pub mod r#trait;
use r#trait::JsonCodec as _;

#[cfg(test)]
mod tests;

/// A codec for JSON
pub struct JsonCodec;

#[async_trait]
impl Codec for JsonCodec {
    fn name(&self) -> &str {
        "json"
    }

    fn status(&self) -> Status {
        Status::Unstable
    }

    fn supported_formats(&self) -> Vec<Format> {
        vec![Format::Json]
    }

    async fn from_str(&self, str: &str, _options: Option<DecodeOptions>) -> Result<(Node, Losses)> {
        let node = Node::from_json(str)?;

        Ok((node, Losses::none()))
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, Losses)> {
        let EncodeOptions { compact, .. } = options.unwrap_or_default();

        let value = node.to_json_value()?;

        let value = if let Some(r#type) = value
            .as_object()
            .and_then(|object| object.get("type"))
            .and_then(|r#type| r#type.as_str())
            .map(String::from)
        {
            let object = value.as_object().expect("checked above").to_owned();

            // Insert the `$schema` and `@context` at the top of the root
            let mut root = Map::with_capacity(object.len() + 1);
            root.insert(
                String::from("$schema"),
                Value::String(format!("https://stencila.dev/{type}.schema.json")),
            );
            for (key, value) in object.into_iter() {
                root.insert(key, value);
            }

            Value::Object(root)
        } else {
            value
        };

        let json = match compact {
            true => serde_json::to_string(&value),
            false => serde_json::to_string_pretty(&value),
        }?;

        Ok((json, Losses::none()))
    }
}
