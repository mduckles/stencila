use std::{collections::BTreeMap, path::Path};

use common::itertools::Itertools;
use formats::Format;
use parser::{
    common::{
        eyre::{bail, Result},
        once_cell::sync::Lazy,
    },
    ParserTrait,
};

// Re-exports
pub use parser::{ParseInfo, Parser, TagMap};

// The following high level functions hide the implementation
// detail of having a static list of parsers. They are intended as the
// only public interface for this crate.

/// Parse some code in a given language
pub fn parse(language: Format, code: &str, path: Option<&Path>) -> Result<ParseInfo> {
    PARSERS.parse(language, code, path)
}

/// List the languages supported by registered parsers
pub fn languages() -> Vec<Format> {
    PARSERS
        .inner
        .values()
        .map(|parser| parser.language)
        .collect_vec()
}

/// The set of registered parsers in the current process
static PARSERS: Lazy<Parsers> = Lazy::new(Parsers::new);

/// A set of registered parsers, either built-in, or provided by plugins
struct Parsers {
    inner: BTreeMap<String, Parser>,
}

/// A macro to dispatch methods to builtin parsers
///
/// This avoids having to do a search over the parsers's specs for matching `languages`.
macro_rules! dispatch_builtins {
    ($format:expr, $method:ident $(,$arg:expr)*) => {
        match $format {
            #[cfg(feature = "parser-bash")]
            Format::Bash | Format::Shell | Format::Zsh => Some(parser_bash::BashParser::$method($($arg),*)),
            #[cfg(feature = "parser-calc")]
            Format::Calc => Some(parser_calc::CalcParser::$method($($arg),*)),
            #[cfg(feature = "parser-http")]
            Format::Http => Some(parser_http::HttpParser::$method($($arg),*)),
            #[cfg(feature = "parser-js")]
            Format::JavaScript => Some(parser_js::JsParser::$method($($arg),*)),
            #[cfg(feature = "parser-json")]
            Format::Json => Some(parser_json::JsonParser::$method($($arg),*)),
            #[cfg(feature = "parser-json5")]
            Format::Json5 => Some(parser_json5::Json5Parser::$method($($arg),*)),
            #[cfg(feature = "parser-postgrest")]
            Format::Postgrest => Some(parser_postgrest::PostgrestParser::$method($($arg),*)),
            #[cfg(feature = "parser-prql")]
            Format::PrQL => Some(parser_prql::PrqlParser::$method($($arg),*)),
            #[cfg(feature = "parser-py")]
            Format::Python => Some(parser_py::PyParser::$method($($arg),*)),
            #[cfg(feature = "parser-r")]
            Format::R => Some(parser_r::RParser::$method($($arg),*)),
            #[cfg(feature = "parser-rust")]
            Format::Rust => Some(parser_rust::RustParser::$method($($arg),*)),
            #[cfg(feature = "parser-sql")]
            Format::SQL => Some(parser_sql::SqlParser::$method($($arg),*)),
            #[cfg(feature = "parser-tailwind")]
            Format::Tailwind => Some(parser_tailwind::TailwindParser::$method($($arg),*)),
            #[cfg(feature = "parser-ts")]
            Format::TypeScript => Some(parser_ts::TsParser::$method($($arg),*)),
            // Fallback to empty result
            _ => Option::<Result<ParseInfo>>::None
        }
    };
}

impl Parsers {
    /// Create a set of parsers
    ///
    /// Note that these strings are labels for the parser which
    /// aim to be consistent with the parser name, are convenient
    /// to use to `stencila parsers show`, and don't need to be
    /// consistent with format names or aliases.
    pub fn new() -> Self {
        let inner = vec![
            #[cfg(feature = "parser-bash")]
            ("bash", parser_bash::BashParser::spec()),
            #[cfg(feature = "parser-calc")]
            ("calc", parser_calc::CalcParser::spec()),
            #[cfg(feature = "parser-http")]
            ("http", parser_http::HttpParser::spec()),
            #[cfg(feature = "parser-js")]
            ("js", parser_js::JsParser::spec()),
            #[cfg(feature = "parser-json")]
            ("json", parser_json::JsonParser::spec()),
            #[cfg(feature = "parser-json5")]
            ("json5", parser_json5::Json5Parser::spec()),
            #[cfg(feature = "parser-postgrest")]
            ("postgrest", parser_postgrest::PostgrestParser::spec()),
            #[cfg(feature = "parser-py")]
            ("prql", parser_prql::PrqlParser::spec()),
            #[cfg(feature = "parser-py")]
            ("py", parser_py::PyParser::spec()),
            #[cfg(feature = "parser-r")]
            ("r", parser_r::RParser::spec()),
            #[cfg(feature = "parser-rust")]
            ("rust", parser_rust::RustParser::spec()),
            #[cfg(feature = "parser-sql")]
            ("sql", parser_sql::SqlParser::spec()),
            #[cfg(feature = "parser-tailwind")]
            ("tailwind", parser_tailwind::TailwindParser::spec()),
            #[cfg(feature = "parser-ts")]
            ("ts", parser_ts::TsParser::spec()),
        ]
        .into_iter()
        .map(|(label, parser): (&str, Parser)| (label.to_string(), parser))
        .collect();

        Self { inner }
    }

    /// List the available parsers
    fn list(&self) -> Vec<String> {
        self.inner
            .keys()
            .into_iter()
            .cloned()
            .collect::<Vec<String>>()
    }

    /// Get a parser by label
    fn get(&self, label: &str) -> Result<Parser> {
        match self.inner.get(label) {
            Some(parser) => Ok(parser.clone()),
            None => bail!("No parser with label `{}`", label),
        }
    }

    /// Parse some code in a language
    fn parse(&self, language: Format, code: &str, path: Option<&Path>) -> Result<ParseInfo> {
        let parse_info = if let Some(result) = dispatch_builtins!(language, parse, code, path) {
            result?
        } else {
            bail!(
                "Unable to parse code in language `{}`: no matching parser found",
                language
            )
        };
        Ok(parse_info)
    }
}

impl Default for Parsers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "cli")]
pub mod commands {
    use std::{fs, path::PathBuf};

    use cli_utils::{
        clap::{self, Parser},
        common::async_trait::async_trait,
        result, Result, Run,
    };

    use super::*;

    /// Manage and use language parsers
    #[derive(Parser)]
    pub struct Command {
        #[clap(subcommand)]
        pub action: Action,
    }

    #[derive(Parser)]
    pub enum Action {
        List(List),
        Show(Show),
        Parse(Parse),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            let Self { action } = self;
            match action {
                Action::List(action) => action.run().await,
                Action::Show(action) => action.run().await,
                Action::Parse(action) => action.run().await,
            }
        }
    }

    /// List the parsers that are available
    ///
    /// The list of available parsers includes those that are built into the Stencila
    /// binary as well as any parsers provided by plugins.
    #[derive(Parser)]
    pub struct List {}
    #[async_trait]
    impl Run for List {
        async fn run(&self) -> Result {
            let list = PARSERS.list();
            result::value(list)
        }
    }

    /// Show the specifications of a parser
    #[derive(Parser)]
    pub struct Show {
        /// The label of the parser
        ///
        /// To get the list of parser labels use `stencila parsers list`.
        label: String,
    }
    #[async_trait]
    impl Run for Show {
        async fn run(&self) -> Result {
            let parser = PARSERS.get(&self.label)?;
            result::value(parser)
        }
    }

    /// Parse some code using a parser
    ///
    /// The code is parsed into a set of graph `Relation`/`Resource` pairs using the
    /// parser that matches the filename extension (or specified using `--lang`).
    /// Useful for testing Stencila's static code analysis for a particular language.
    #[derive(Parser)]
    pub struct Parse {
        /// The file (or code) to parse
        #[clap(multiple_values = true)]
        code: Vec<String>,

        /// If the argument should be treated as text, rather than a file path
        #[clap(short, long)]
        text: bool,

        /// The language of the code
        #[clap(short, long)]
        lang: Option<String>,
    }
    #[async_trait]
    impl Run for Parse {
        async fn run(&self) -> Result {
            let (path, code, lang) = if self.text || self.code.len() > 1 {
                let code = self.code.join(" ");
                (
                    "<text>".to_string(),
                    code,
                    self.lang.clone().unwrap_or_default(),
                )
            } else {
                let file = self.code[0].clone();
                let code = fs::read_to_string(&file)?;
                let ext = PathBuf::from(&file)
                    .extension()
                    .map(|ext| ext.to_string_lossy().to_string())
                    .unwrap_or_default();
                let lang = self.lang.clone().or(Some(ext)).unwrap_or_default();
                (file, code, lang)
            };

            let language = formats::match_name(&lang);
            let path = PathBuf::from(path);
            let parse_info = PARSERS.parse(language, &code, Some(&path))?;
            result::value(parse_info)
        }
    }
}
