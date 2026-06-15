use std::{fmt, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfTextExtractionError {
    Encrypted,
    Invalid(String),
}

impl fmt::Display for PdfTextExtractionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encrypted => formatter.write_str("PDF is encrypted"),
            Self::Invalid(message) => formatter.write_str(message),
        }
    }
}

pub trait PdfTextExtractor: Send + Sync {
    fn extract_text(&self, path: &Path) -> Result<String, PdfTextExtractionError>;
}
