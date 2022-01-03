use eyre::Result;
use graph_triples::{Pairs, Relation};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::path::Path;

// Export and re-export for the convenience of crates that implement a parser
pub mod utils;
pub use eyre;
pub use formats;
pub use graph_triples;

/// A specification for parsers
///
/// All parsers, including those implemented in plugins, should provide this
/// specification. Rust implementations return a `Parser` instance from the
/// `spec` function of `ParserTrait`. Plugins provide a JSON or YAML serialization
/// as part of their manifest.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Parser {
    /// The language that the parser parses
    pub language: String,
}

/// A trait for parsers
///
/// This trait can be used by Rust implementations of parsers, allowing them to
/// be compiled into the Stencila binaries.
///
/// It defines similar functions to `serde_json` (and other `serde_` crates) for
/// converting nodes to/from strings, files, readers etc.
pub trait ParserTrait {
    /// Get the [`Parser`] specification
    fn spec() -> Parser;

    /// Parse some code and return a set of graph pairs
    fn parse(path: &Path, code: &str) -> Result<ParseInfo>;
}

/// The result of parsing
#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct ParseInfo {
    /// Whether the code had an explicit `@pure` or `@impure` tag
    pub pure: Option<bool>,

    /// The [`Relation`]-[`Resource`] pairs between the code and other resources
    /// (e.g. `Symbol`s, `File`s)
    pub relations: Pairs,
}

impl ParseInfo {
    /// Is the parse code pure (i.e. has no side effects)?
    ///
    /// If the code has not been explicitly tagged as `@pure` or `@impure` then
    /// returns `true` if there are any side-effect causing relations.
    pub fn is_pure(&self) -> bool {
        self.pure.unwrap_or_else(|| {
            self.relations
                .iter()
                .filter(|(relation, ..)| {
                    matches!(
                        relation,
                        Relation::Assign(..)
                            | Relation::Alter(..)
                            | Relation::Import(..)
                            | Relation::Write(..)
                    )
                })
                .count()
                == 0
        })
    }
}
