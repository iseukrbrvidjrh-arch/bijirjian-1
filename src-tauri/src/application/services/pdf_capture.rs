use std::{fs, path::Path};

use serde::Serialize;

use crate::{
    domain::{
        ports::{PdfTextExtractionError, PdfTextExtractor, SourceRepository, WorkspaceRepository},
        Source,
    },
    error::AppError,
};

const MAX_PDF_FILE_SIZE: u64 = 20 * 1024 * 1024;
const MAX_EXTRACTED_TEXT_CHARACTERS: usize = 200_000;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PdfSourceMetadata<'metadata> {
    original_file_name: &'metadata str,
    file_size: u64,
    extracted_text_length: usize,
    captured_via: &'static str,
}

pub trait PdfCaptureService: Send + Sync {
    fn capture_pdf_source(&self, file_path: String) -> Result<Source, AppError>;
}

pub struct DefaultPdfCaptureService<'service, WorkspaceRepo, SourceRepo, Extractor>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SourceRepo: SourceRepository + ?Sized,
    Extractor: PdfTextExtractor + ?Sized,
{
    workspace_repository: &'service WorkspaceRepo,
    source_repository: &'service SourceRepo,
    extractor: &'service Extractor,
}

impl<'service, WorkspaceRepo, SourceRepo, Extractor>
    DefaultPdfCaptureService<'service, WorkspaceRepo, SourceRepo, Extractor>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SourceRepo: SourceRepository + ?Sized,
    Extractor: PdfTextExtractor + ?Sized,
{
    pub const fn new(
        workspace_repository: &'service WorkspaceRepo,
        source_repository: &'service SourceRepo,
        extractor: &'service Extractor,
    ) -> Self {
        Self {
            workspace_repository,
            source_repository,
            extractor,
        }
    }
}

impl<WorkspaceRepo, SourceRepo, Extractor> PdfCaptureService
    for DefaultPdfCaptureService<'_, WorkspaceRepo, SourceRepo, Extractor>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SourceRepo: SourceRepository + ?Sized,
    Extractor: PdfTextExtractor + ?Sized,
{
    fn capture_pdf_source(&self, file_path: String) -> Result<Source, AppError> {
        let file_path = file_path.trim();
        if file_path.is_empty() {
            return Err(AppError::Validation(
                "PDF file path must not be empty".to_owned(),
            ));
        }

        let path = Path::new(file_path);
        let metadata = fs::metadata(path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                AppError::Validation(format!("PDF file does not exist: {file_path}"))
            } else {
                AppError::Validation(format!("PDF file could not be accessed: {file_path}"))
            }
        })?;
        if !metadata.is_file() {
            return Err(AppError::Validation(format!(
                "PDF path is not a file: {file_path}"
            )));
        }
        if !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
        {
            return Err(AppError::Validation(
                "selected file must have a .pdf extension".to_owned(),
            ));
        }
        if metadata.len() > MAX_PDF_FILE_SIZE {
            return Err(AppError::Validation(format!(
                "PDF file exceeds the 20 MiB limit: {} bytes",
                metadata.len()
            )));
        }

        let extracted_text = self
            .extractor
            .extract_text(path)
            .map_err(map_extraction_error)?;
        let extracted_text = extracted_text.trim();
        if extracted_text.is_empty() {
            return Err(AppError::Validation(
                "This PDF has no extractable text. OCR is not supported in the current version"
                    .to_owned(),
            ));
        }
        let extracted_text_length = extracted_text.chars().count();
        if extracted_text_length > MAX_EXTRACTED_TEXT_CHARACTERS {
            return Err(AppError::Validation(format!(
                "extracted PDF text exceeds the 200,000 character limit: {extracted_text_length} characters"
            )));
        }
        let original_file_name = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .filter(|name| !name.is_empty())
            .ok_or_else(|| {
                AppError::Validation("PDF file name could not be determined".to_owned())
            })?;
        let metadata_json = serde_json::to_string(&PdfSourceMetadata {
            original_file_name: original_file_name.as_ref(),
            file_size: metadata.len(),
            extracted_text_length,
            captured_via: "pdf",
        })
        .map_err(|error| {
            AppError::State(format!(
                "PDF source metadata could not be serialized: {error}"
            ))
        })?;

        let workspace = self.workspace_repository.ensure_default_workspace()?;
        self.source_repository
            .insert_pdf_source(&workspace.id, extracted_text, &metadata_json)
    }
}

fn map_extraction_error(error: PdfTextExtractionError) -> AppError {
    match error {
        PdfTextExtractionError::Encrypted => AppError::Validation(
            "This PDF is encrypted and cannot be imported without a password".to_owned(),
        ),
        PdfTextExtractionError::Invalid(message) => {
            AppError::Validation(format!("PDF is damaged or could not be parsed: {message}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        path::{Path, PathBuf},
    };

    use rusqlite::Connection;
    use sha2::Digest;

    use super::{
        DefaultPdfCaptureService, PdfCaptureService, MAX_EXTRACTED_TEXT_CHARACTERS,
        MAX_PDF_FILE_SIZE,
    };
    use crate::{
        domain::{
            ports::{
                PdfTextExtractionError, PdfTextExtractor, SourceRepository, WorkspaceRepository,
            },
            InboxStatus, SourceType,
        },
        error::AppError,
        infrastructure::database::{
            repositories::{SqliteSourceRepository, SqliteWorkspaceRepository},
            Database,
        },
    };

    #[test]
    fn captures_pdf_text_and_safe_metadata_in_the_default_workspace() {
        let fixture = FileFixture::new("research.PDF", b"%PDF");
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let extractor = FakeExtractor::text("  Extracted local-first knowledge.  ");
        let service =
            DefaultPdfCaptureService::new(&workspace_repository, &source_repository, &extractor);

        let source = service
            .capture_pdf_source(fixture.path_string())
            .expect("capture PDF source");
        let default_workspace = workspace_repository
            .find_default_workspace()
            .expect("find default workspace")
            .expect("default workspace should exist");
        let metadata: serde_json::Value = serde_json::from_str(
            source
                .metadata_json
                .as_deref()
                .expect("PDF metadata should exist"),
        )
        .expect("parse PDF metadata");

        assert_eq!(source.workspace_id, default_workspace.id);
        assert_eq!(source.source_type, SourceType::Pdf);
        assert_eq!(source.inbox_status, InboxStatus::Unprocessed);
        assert_eq!(source.raw_content, "Extracted local-first knowledge.");
        assert_eq!(
            source.content_hash,
            format!("{:x}", sha2::Sha256::digest(source.raw_content.as_bytes()))
        );
        assert_eq!(metadata["originalFileName"], "research.PDF");
        assert_eq!(metadata["fileSize"], 4);
        assert_eq!(metadata["extractedTextLength"], 32);
        assert_eq!(metadata["capturedVia"], "pdf");
        assert!(metadata.get("originalFilePath").is_none());
        assert!(!source
            .metadata_json
            .as_deref()
            .is_some_and(|value| value.contains(fixture.path.to_string_lossy().as_ref())));

        let inbox = source_repository
            .list_inbox_sources(&default_workspace.id, Some("LOCAL-FIRST"), 50)
            .expect("search PDF source in inbox");
        assert_eq!(inbox, vec![source]);
    }

    #[test]
    fn rejects_invalid_paths_extensions_and_oversized_files_before_extraction() {
        let fixture = FileFixture::new("notes.txt", b"not a PDF");
        let oversized = FileFixture::sparse("large.pdf", MAX_PDF_FILE_SIZE + 1);
        let directory = FileFixture::directory();
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let extractor = FakeExtractor::text("unused");
        let service =
            DefaultPdfCaptureService::new(&workspace_repository, &source_repository, &extractor);

        for result in [
            service.capture_pdf_source(" \n".to_owned()),
            service.capture_pdf_source(directory.path.join("missing.pdf").to_string_lossy().into()),
            service.capture_pdf_source(directory.path_string()),
            service.capture_pdf_source(fixture.path_string()),
            service.capture_pdf_source(oversized.path_string()),
        ] {
            assert!(matches!(result, Err(AppError::Validation(_))));
        }
        assert_eq!(source_count(&database), 0);
        assert_eq!(extractor.call_count(), 0);
    }

    #[test]
    fn rejects_empty_encrypted_invalid_and_excessive_extraction_without_writing() {
        let fixture = FileFixture::new("valid.pdf", b"%PDF");

        for extractor in [
            FakeExtractor::text(" \n\t "),
            FakeExtractor::encrypted(),
            FakeExtractor::invalid("broken xref"),
            FakeExtractor::text(&"a".repeat(MAX_EXTRACTED_TEXT_CHARACTERS + 1)),
        ] {
            let database = test_database();
            let workspace_repository = SqliteWorkspaceRepository::new(&database);
            let source_repository = SqliteSourceRepository::new(&database);
            let service = DefaultPdfCaptureService::new(
                &workspace_repository,
                &source_repository,
                &extractor,
            );

            assert!(matches!(
                service.capture_pdf_source(fixture.path_string()),
                Err(AppError::Validation(_))
            ));
            assert_eq!(source_count(&database), 0);
        }
    }

    #[test]
    fn pdf_source_remains_readable_by_existing_source_consumers() {
        let fixture = FileFixture::new("summary.pdf", b"%PDF");
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let extractor = FakeExtractor::text("PDF content for the existing Summary Service");
        let service =
            DefaultPdfCaptureService::new(&workspace_repository, &source_repository, &extractor);

        let source = service
            .capture_pdf_source(fixture.path_string())
            .expect("capture PDF source");
        let found = source_repository
            .find_source(&source.workspace_id, &source.id)
            .expect("existing source consumers can read PDF source");

        assert_eq!(found.source_type, SourceType::Pdf);
        assert_eq!(
            found.raw_content,
            "PDF content for the existing Summary Service"
        );
    }

    struct FakeExtractor {
        result: Result<String, PdfTextExtractionError>,
        calls: std::sync::Mutex<usize>,
    }

    impl FakeExtractor {
        fn text(text: &str) -> Self {
            Self {
                result: Ok(text.to_owned()),
                calls: std::sync::Mutex::new(0),
            }
        }

        fn encrypted() -> Self {
            Self {
                result: Err(PdfTextExtractionError::Encrypted),
                calls: std::sync::Mutex::new(0),
            }
        }

        fn invalid(message: &str) -> Self {
            Self {
                result: Err(PdfTextExtractionError::Invalid(message.to_owned())),
                calls: std::sync::Mutex::new(0),
            }
        }

        fn call_count(&self) -> usize {
            *self.calls.lock().expect("lock fake extractor calls")
        }
    }

    impl PdfTextExtractor for FakeExtractor {
        fn extract_text(&self, _path: &Path) -> Result<String, PdfTextExtractionError> {
            *self.calls.lock().expect("lock fake extractor calls") += 1;
            self.result.clone()
        }
    }

    struct FileFixture {
        path: PathBuf,
        is_directory: bool,
    }

    impl FileFixture {
        fn new(name: &str, contents: &[u8]) -> Self {
            let directory = fixture_directory();
            let path = directory.join(name);
            fs::write(&path, contents).expect("write file fixture");
            Self {
                path,
                is_directory: false,
            }
        }

        fn sparse(name: &str, length: u64) -> Self {
            let directory = fixture_directory();
            let path = directory.join(name);
            File::create(&path)
                .expect("create sparse fixture")
                .set_len(length)
                .expect("size sparse fixture");
            Self {
                path,
                is_directory: false,
            }
        }

        fn directory() -> Self {
            Self {
                path: fixture_directory(),
                is_directory: true,
            }
        }

        fn path_string(&self) -> String {
            self.path.to_string_lossy().into_owned()
        }
    }

    impl Drop for FileFixture {
        fn drop(&mut self) {
            if self.is_directory {
                let _ = fs::remove_dir_all(&self.path);
            } else if let Some(parent) = self.path.parent() {
                let _ = fs::remove_dir_all(parent);
            }
        }
    }

    fn fixture_directory() -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("second-brain-os-pdf-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&directory).expect("create fixture directory");
        directory
    }

    fn source_count(database: &Database) -> i64 {
        database
            .with_connection(|connection| {
                connection
                    .query_row("SELECT COUNT(*) FROM sources", [], |row| row.get(0))
                    .map_err(AppError::from)
            })
            .expect("count sources")
    }

    fn test_database() -> Database {
        Database::from_connection(Connection::open_in_memory().expect("open in-memory database"))
            .expect("initialize test database")
    }
}
