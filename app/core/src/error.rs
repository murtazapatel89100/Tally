use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("{message}")]
pub struct ParseError {
    pub message: String,
    #[source_code]
    pub src: NamedSource<String>,
    #[label("{}", self.label)]
    pub span: SourceSpan,
    pub label: String,
    #[help]
    pub help: Option<String>,
}

impl ParseError {
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
