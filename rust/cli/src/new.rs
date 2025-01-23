use std::path::PathBuf;

use common::{
    clap::{self, Parser, ValueEnum},
    eyre::Result,
};
use document::Document;
use format::Format;
use schema::NodeType;
use color_print::cstr;

pub static AFTER_HELP: &str = cstr!("
<bold>Usage Instructions</bold>

<bold>Example:</bold> <cyan>`stencila new article.smd`</cyan>

Detailed usage information provided in long-form help page,
available by <cyan>`stencila new --help`</cyan>
");

pub static AFTER_LONG_HELP: &str = cstr!("
<bold> Further Information and Examples </bold>

<bold>Examples:</bold><cyan>
        `stencila new article.smd --type article --sidecar json.zip`
        `stencila new article.docx --sidecar json`
        `stencila new article.md`
    </cyan>

    The File extension must be one of the supported codecs which can be found using <cyan>`stencila codecs list`</cyan>
");
/// Create a new document with sidecar file
#[derive(Debug, Parser)]
pub struct Cli {
    /// The path of the document to create with a supported extension 
    #[arg(value_name="File")]
    path: PathBuf,

    /// Overwrite the document, and any sidecar file, if they already exist
    #[arg(long, short)]
    force: bool,

    /// The type of document to create
    #[arg(long, short, default_value = "article")]
    r#type: RootType,

    /// The format of the sidecar file
    #[arg(long, short)]
    sidecar: Option<SidecarFormat>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum RootType {
    #[clap(alias = "article")]
    Article,
    #[clap(alias = "prompt")]
    Prompt,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum SidecarFormat {
    #[clap(name = "json.zip", alias = "json-zip")]
    JsonZip,
    Json,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let node_type = match self.r#type {
            RootType::Article => NodeType::Article,
            RootType::Prompt => NodeType::Prompt,
        };
        let sidecar = self.sidecar.map(|format| match format {
            SidecarFormat::JsonZip => Format::JsonZip,
            SidecarFormat::Json => Format::Json,
        });
        Document::create(&self.path, self.force, node_type, sidecar).await?;

        Ok(())
    }
}
