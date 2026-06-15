use std::{panic::AssertUnwindSafe, path::Path};

use pdf_extract::OutputError;

use crate::domain::ports::{PdfTextExtractionError, PdfTextExtractor};

pub struct PdfExtractAdapter;

impl PdfExtractAdapter {
    pub const fn new() -> Self {
        Self
    }
}

impl Default for PdfExtractAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfTextExtractor for PdfExtractAdapter {
    fn extract_text(&self, path: &Path) -> Result<String, PdfTextExtractionError> {
        match std::panic::catch_unwind(AssertUnwindSafe(|| pdf_extract::extract_text(path))) {
            Ok(Ok(text)) => Ok(text),
            Ok(Err(OutputError::PdfError(pdf_extract::Error::Decryption(_)))) => {
                Err(PdfTextExtractionError::Encrypted)
            }
            Ok(Err(error)) => Err(PdfTextExtractionError::Invalid(error.to_string())),
            Err(_) => Err(PdfTextExtractionError::Invalid(
                "PDF parser failed while reading the document".to_owned(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::PdfExtractAdapter;
    use crate::domain::ports::PdfTextExtractor;

    #[test]
    fn extracts_text_from_a_minimal_pdf_fixture() {
        let fixture = PdfFixture::new("Hello PDF");
        let extracted = PdfExtractAdapter::new()
            .extract_text(&fixture.path)
            .expect("extract fixture text");

        assert!(extracted.contains("Hello PDF"));
    }

    struct PdfFixture {
        path: PathBuf,
    }

    impl PdfFixture {
        fn new(text: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "second-brain-os-pdf-extract-{}.pdf",
                uuid::Uuid::new_v4()
            ));
            fs::write(&path, minimal_pdf(text)).expect("write PDF fixture");
            Self { path }
        }
    }

    impl Drop for PdfFixture {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    fn minimal_pdf(text: &str) -> Vec<u8> {
        let escaped = text
            .replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)");
        let stream = format!("BT /F1 12 Tf 72 720 Td ({escaped}) Tj ET");
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_owned(),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
            format!("<< /Length {} >>\nstream\n{stream}\nendstream", stream.len()),
        ];
        let mut pdf = b"%PDF-1.4\n".to_vec();
        let mut offsets = Vec::new();

        for (index, object) in objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.extend_from_slice(format!("{} 0 obj\n{object}\nendobj\n", index + 1).as_bytes());
        }

        let xref_offset = pdf.len();
        pdf.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for offset in offsets {
            pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
                objects.len() + 1
            )
            .as_bytes(),
        );
        pdf
    }
}
