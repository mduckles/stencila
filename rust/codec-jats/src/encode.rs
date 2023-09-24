use codec::{common::eyre::Result, schema::Node, EncodeOptions, LossDirection, Losses};
use codec_jats_trait::JatsCodec as _;

pub(super) fn encode(node: &Node, options: Option<EncodeOptions>) -> Result<(String, Losses)> {
    let EncodeOptions { compact, .. } = options.unwrap_or_default();

    if !matches!(node, Node::Article(..)) {
        return Ok((
            String::new(),
            Losses::of_everything(LossDirection::Encode, node.to_string()),
        ));
    }

    let (mut jats, losses) = node.to_jats();
    if !compact {
        jats = indent(&jats);
    }

    Ok((jats, losses))
}

/// Indent JATS
///
/// Originally based on https://gist.github.com/lwilli/14fb3178bd9adac3a64edfbc11f42e0d
fn indent(jats: &str) -> String {
    use quick_xml::{events::Event, Reader, Writer};

    let mut reader = Reader::from_str(jats);
    reader.trim_text(true);

    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(event) => writer.write_event(event),
            Err(error) => panic!(
                "Error at position {}: {:?}",
                reader.buffer_position(),
                error
            ),
        }
        .expect("Failed to parse XML");
    }

    std::str::from_utf8(&writer.into_inner())
        .expect("Failed to convert a slice of bytes to a string slice")
        .to_string()
}