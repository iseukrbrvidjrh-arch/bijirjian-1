use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeType {
    Concept,
    Tool,
    Project,
    Question,
    Solution,
    Insight,
    Resource,
    Person,
}

impl KnowledgeType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Concept => "concept",
            Self::Tool => "tool",
            Self::Project => "project",
            Self::Question => "question",
            Self::Solution => "solution",
            Self::Insight => "insight",
            Self::Resource => "resource",
            Self::Person => "person",
        }
    }
}

impl TryFrom<&str> for KnowledgeType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "concept" => Ok(Self::Concept),
            "tool" => Ok(Self::Tool),
            "project" => Ok(Self::Project),
            "question" => Ok(Self::Question),
            "solution" => Ok(Self::Solution),
            "insight" => Ok(Self::Insight),
            "resource" => Ok(Self::Resource),
            "person" => Ok(Self::Person),
            _ => Err(format!("unsupported knowledge type: {value}")),
        }
    }
}

impl fmt::Display for KnowledgeType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeStatus {
    Proposed,
    Accepted,
    Archived,
}

impl KnowledgeStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for KnowledgeStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "proposed" => Ok(Self::Proposed),
            "accepted" => Ok(Self::Accepted),
            "archived" => Ok(Self::Archived),
            _ => Err(format!("unsupported knowledge status: {value}")),
        }
    }
}

impl fmt::Display for KnowledgeStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeNode {
    pub id: String,
    pub workspace_id: String,
    pub ai_run_id: Option<String>,
    pub title: String,
    pub content: String,
    pub knowledge_type: KnowledgeType,
    pub status: KnowledgeStatus,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{KnowledgeStatus, KnowledgeType};

    #[test]
    fn parses_supported_knowledge_types() {
        let supported = [
            ("concept", KnowledgeType::Concept),
            ("tool", KnowledgeType::Tool),
            ("project", KnowledgeType::Project),
            ("question", KnowledgeType::Question),
            ("solution", KnowledgeType::Solution),
            ("insight", KnowledgeType::Insight),
            ("resource", KnowledgeType::Resource),
            ("person", KnowledgeType::Person),
        ];

        for (value, expected) in supported {
            assert_eq!(KnowledgeType::try_from(value), Ok(expected));
        }
        assert!(KnowledgeType::try_from("note").is_err());
    }

    #[test]
    fn parses_supported_knowledge_statuses() {
        assert_eq!(
            KnowledgeStatus::try_from("proposed"),
            Ok(KnowledgeStatus::Proposed)
        );
        assert_eq!(
            KnowledgeStatus::try_from("accepted"),
            Ok(KnowledgeStatus::Accepted)
        );
        assert_eq!(
            KnowledgeStatus::try_from("archived"),
            Ok(KnowledgeStatus::Archived)
        );
        assert!(KnowledgeStatus::try_from("rejected").is_err());
    }
}
