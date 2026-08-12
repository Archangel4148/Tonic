//! Import warnings. Parsers never discard usable content.

/// User-facing summary when any import warning was produced.
pub const UNRECOGNIZED_CONTENT_MESSAGE: &str = "Some content could not be recognized.";

/// User-facing summary when only unsupported MusicXML features were skipped.
pub const UNSUPPORTED_MUSICXML_MESSAGE: &str = "Some MusicXML features are not supported.";

/// Kind of recoverable import problem.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WarningKind {
    UnrecognizedChord,
    PartialChord,
    UnrecognizedDirective,
    MalformedInput,
    AmbiguousLayout,
    SkippedContent,
    UnsupportedFeature,
}

/// A non-fatal issue found while importing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportWarning {
    pub kind: WarningKind,
    pub message: String,
    /// 1-based line number in the source text, when known.
    pub line: Option<u32>,
}

impl ImportWarning {
    #[must_use]
    pub fn new(kind: WarningKind, message: impl Into<String>, line: Option<u32>) -> Self {
        Self {
            kind,
            message: message.into(),
            line,
        }
    }
}
