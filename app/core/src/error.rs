//! Friendly, source-spanned parse diagnostics.
//!
//! [`ParseError`] implements [`miette::Diagnostic`], so when it is printed by a
//! miette-aware reporter (the `tally` binary in Phase 3, or `{:?}` with the
//! `fancy` feature) it renders the file name, the offending line/column, and a
//! caret underlining the exact token — never a bare "parse error".

use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

/// A single parse failure with enough context to point a caret at the problem.
#[derive(Debug, Error, Diagnostic)]
#[error("{message}")]
pub struct ParseError {
    /// The headline shown next to the error (e.g. `invalid amount`).
    pub message: String,
    /// The full journal source, so the reporter can render the surrounding lines.
    #[source_code]
    pub src: NamedSource<String>,
    /// Byte offset + length of the offending token within `src`.
    #[label("{}", self.label)]
    pub span: SourceSpan,
    /// The text placed under the caret.
    pub label: String,
    /// An optional hint on how to fix it.
    #[help]
    pub help: Option<String>,
}

impl ParseError {
    /// Build a diagnostic. `span` is `(byte_offset, byte_length)`.
    pub fn new(
        name: impl AsRef<str>,
        source: impl Into<String>,
        span: impl Into<SourceSpan>,
        message: impl Into<String>,
        label: impl Into<String>,
        help: Option<String>,
    ) -> Self {
        Self {
            message: message.into(),
            src: NamedSource::new(name, source.into()).with_language("ledger"),
            span: span.into(),
            label: label.into(),
            help,
        }
    }
}
