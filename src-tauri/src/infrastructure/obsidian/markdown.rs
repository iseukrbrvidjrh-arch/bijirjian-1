use std::{
    fs,
    path::{Path, PathBuf},
};

use uuid::Uuid;

use crate::{
    domain::{ports::KnowledgeMarkdownWriter, KnowledgeNode},
    error::AppError,
};

const EXPORT_DIRECTORY: [&str; 2] = ["SecondBrainOS", "Knowledge"];
const MAX_SLUG_CHARACTERS: usize = 64;

pub struct FileSystemKnowledgeMarkdownWriter;

impl FileSystemKnowledgeMarkdownWriter {
    pub const fn new() -> Self {
        Self
    }
}

impl Default for FileSystemKnowledgeMarkdownWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeMarkdownWriter for FileSystemKnowledgeMarkdownWriter {
    fn write_markdown(
        &self,
        vault_path: &str,
        knowledge: &KnowledgeNode,
    ) -> Result<String, AppError> {
        let vault_path = Path::new(vault_path);
        let metadata = fs::metadata(vault_path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                AppError::Validation(format!(
                    "Obsidian vault path does not exist: {}",
                    vault_path.display()
                ))
            } else {
                AppError::Validation(format!(
                    "Obsidian vault path could not be accessed: {}",
                    vault_path.display()
                ))
            }
        })?;
        if !metadata.is_dir() {
            return Err(AppError::Validation(format!(
                "Obsidian vault path is not a directory: {}",
                vault_path.display()
            )));
        }

        let export_directory = export_directory(vault_path);
        fs::create_dir_all(&export_directory)?;

        let target_path = export_directory.join(filename_for(knowledge));
        let temporary_path = export_directory.join(format!(
            ".{}.{}.tmp",
            target_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("knowledge.md"),
            Uuid::new_v4()
        ));
        let markdown = render_markdown(knowledge)?;

        fs::write(&temporary_path, markdown)?;
        if let Err(error) = fs::rename(&temporary_path, &target_path) {
            let _ = fs::remove_file(&temporary_path);
            return Err(AppError::Io(error));
        }

        Ok(target_path.to_string_lossy().into_owned())
    }
}

fn export_directory(vault_path: &Path) -> PathBuf {
    EXPORT_DIRECTORY
        .iter()
        .fold(vault_path.to_path_buf(), |path, component| {
            path.join(component)
        })
}

fn filename_for(knowledge: &KnowledgeNode) -> String {
    let slug = slugify(&knowledge.title);
    let short_id = knowledge
        .id
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .take(8)
        .collect::<String>()
        .to_ascii_lowercase();
    let short_id = if short_id.is_empty() {
        "node"
    } else {
        short_id.as_str()
    };

    format!("{slug}–{short_id}.md")
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    let mut pending_separator = false;
    let mut character_count = 0;

    for character in title.chars() {
        if character.is_alphanumeric() {
            if pending_separator && !slug.is_empty() && character_count < MAX_SLUG_CHARACTERS {
                slug.push('-');
                character_count += 1;
            }
            pending_separator = false;

            for lowercase in character.to_lowercase() {
                if character_count >= MAX_SLUG_CHARACTERS {
                    break;
                }
                slug.push(lowercase);
                character_count += 1;
            }
        } else if !slug.is_empty() {
            pending_separator = true;
        }

        if character_count >= MAX_SLUG_CHARACTERS {
            break;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        "knowledge".to_owned()
    } else {
        slug
    }
}

fn render_markdown(knowledge: &KnowledgeNode) -> Result<String, AppError> {
    let title = collapse_whitespace(&knowledge.title);
    let title_yaml = yaml_string(&title)?;
    let id_yaml = yaml_string(&knowledge.id)?;
    let workspace_id_yaml = yaml_string(&knowledge.workspace_id)?;
    let knowledge_type_yaml = yaml_string(knowledge.knowledge_type.as_str())?;
    let status_yaml = yaml_string(knowledge.status.as_str())?;
    let created_at_yaml = yaml_string(&knowledge.created_at)?;
    let updated_at_yaml = yaml_string(&knowledge.updated_at)?;
    let type_label = display_label(knowledge.knowledge_type.as_str());
    let status_label = display_label(knowledge.status.as_str());

    Ok(format!(
        "---\n\
         title: {title_yaml}\n\
         knowledge_id: {id_yaml}\n\
         workspace_id: {workspace_id_yaml}\n\
         knowledge_type: {knowledge_type_yaml}\n\
         status: {status_yaml}\n\
         created_at: {created_at_yaml}\n\
         updated_at: {updated_at_yaml}\n\
         ---\n\n\
         # {title}\n\n\
         **Type:** {type_label}  \n\
         **Status:** {status_label}  \n\
         **Created:** {}  \n\
         **Updated:** {}\n\n\
         {}\n",
        knowledge.created_at,
        knowledge.updated_at,
        knowledge.content.trim()
    ))
}

fn yaml_string(value: &str) -> Result<String, AppError> {
    serde_json::to_string(value)
        .map_err(|error| AppError::State(format!("failed to encode Markdown frontmatter: {error}")))
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn display_label(value: &str) -> String {
    let mut characters = value.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{filename_for, render_markdown, slugify, FileSystemKnowledgeMarkdownWriter};
    use crate::domain::{
        ports::KnowledgeMarkdownWriter, KnowledgeNode, KnowledgeStatus, KnowledgeType,
    };

    #[test]
    fn sanitizes_filenames_and_uses_an_id_suffix() {
        let mut knowledge = test_knowledge("Local / First: Architecture?");

        assert_eq!(
            filename_for(&knowledge),
            "local-first-architecture–12345678.md"
        );

        let mut same_title = knowledge.clone();
        same_title.id = "abcdef12-abcd-efab-cdef-1234567890ab".to_owned();
        assert_eq!(
            filename_for(&same_title),
            "local-first-architecture–abcdef12.md"
        );
        assert_ne!(filename_for(&knowledge), filename_for(&same_title));

        knowledge.title = "///".to_owned();
        assert_eq!(filename_for(&knowledge), "knowledge–12345678.md");

        let long_slug = slugify(&"A".repeat(100));
        assert_eq!(long_slug.chars().count(), 64);
    }

    #[test]
    fn renders_safe_frontmatter_and_plain_markdown_content() {
        let knowledge = test_knowledge("Local \"First\"\nArchitecture");

        let markdown = render_markdown(&knowledge).expect("render Markdown");

        assert!(markdown.contains("title: \"Local \\\"First\\\" Architecture\""));
        assert!(markdown.contains("knowledge_id: \"12345678-abcd-efab-cdef-1234567890ab\""));
        assert!(markdown.contains("knowledge_type: \"concept\""));
        assert!(markdown.contains("status: \"accepted\""));
        assert!(markdown.contains("# Local \"First\" Architecture"));
        assert!(markdown.contains("**Type:** Concept"));
        assert!(markdown.contains("Knowledge content."));
        assert!(!markdown.contains("api_key"));
        assert!(!markdown.contains("raw_response"));
    }

    #[test]
    fn writes_to_the_fixed_directory_and_overwrites_the_same_file() {
        let fixture = VaultFixture::new();
        let writer = FileSystemKnowledgeMarkdownWriter::new();
        let mut knowledge = test_knowledge("Local First");

        let first_path = writer
            .write_markdown(&fixture.path_string(), &knowledge)
            .expect("write first export");
        knowledge.content = "Updated knowledge content.".to_owned();
        let second_path = writer
            .write_markdown(&fixture.path_string(), &knowledge)
            .expect("overwrite export");

        assert_eq!(first_path, second_path);
        assert!(first_path.contains("SecondBrainOS/Knowledge"));
        assert!(fs::read_to_string(second_path)
            .expect("read overwritten Markdown")
            .contains("Updated knowledge content."));
        assert_eq!(
            fs::read_dir(fixture.path.join("SecondBrainOS/Knowledge"))
                .expect("read export directory")
                .count(),
            1
        );
    }

    fn test_knowledge(title: &str) -> KnowledgeNode {
        KnowledgeNode {
            id: "12345678-abcd-efab-cdef-1234567890ab".to_owned(),
            workspace_id: "workspace-1".to_owned(),
            ai_run_id: None,
            title: title.to_owned(),
            content: "Knowledge content.".to_owned(),
            knowledge_type: KnowledgeType::Concept,
            status: KnowledgeStatus::Accepted,
            created_at: "2026-06-15T00:00:00.000Z".to_owned(),
            updated_at: "2026-06-15T01:00:00.000Z".to_owned(),
            archived_at: None,
        }
    }

    struct VaultFixture {
        path: PathBuf,
    }

    impl VaultFixture {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should follow Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "second-brain-os-markdown-writer-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("create vault fixture");
            Self { path }
        }

        fn path_string(&self) -> String {
            self.path.to_string_lossy().into_owned()
        }
    }

    impl Drop for VaultFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
