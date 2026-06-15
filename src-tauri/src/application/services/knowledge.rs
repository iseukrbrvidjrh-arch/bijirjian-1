use crate::{
    domain::{
        ports::{KnowledgeRepository, WorkspaceRepository},
        KnowledgeNode, KnowledgeStatus, KnowledgeType,
    },
    error::AppError,
};

pub trait KnowledgeService: Send + Sync {
    fn create_knowledge_node(
        &self,
        title: String,
        content: String,
        knowledge_type: String,
    ) -> Result<KnowledgeNode, AppError>;

    fn accept_knowledge_node(&self, knowledge_id: String) -> Result<KnowledgeNode, AppError>;

    fn archive_knowledge_node(&self, knowledge_id: String) -> Result<KnowledgeNode, AppError>;

    fn list_knowledge_nodes(
        &self,
        status: Option<String>,
        knowledge_type: Option<String>,
        query: Option<String>,
        limit: usize,
    ) -> Result<Vec<KnowledgeNode>, AppError>;
}

pub struct DefaultKnowledgeService<'service, WorkspaceRepo, KnowledgeRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    KnowledgeRepo: KnowledgeRepository + ?Sized,
{
    workspace_repository: &'service WorkspaceRepo,
    knowledge_repository: &'service KnowledgeRepo,
}

impl<'service, WorkspaceRepo, KnowledgeRepo>
    DefaultKnowledgeService<'service, WorkspaceRepo, KnowledgeRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    KnowledgeRepo: KnowledgeRepository + ?Sized,
{
    pub const fn new(
        workspace_repository: &'service WorkspaceRepo,
        knowledge_repository: &'service KnowledgeRepo,
    ) -> Self {
        Self {
            workspace_repository,
            knowledge_repository,
        }
    }
}

impl<WorkspaceRepo, KnowledgeRepo> KnowledgeService
    for DefaultKnowledgeService<'_, WorkspaceRepo, KnowledgeRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    KnowledgeRepo: KnowledgeRepository + ?Sized,
{
    fn create_knowledge_node(
        &self,
        title: String,
        content: String,
        knowledge_type: String,
    ) -> Result<KnowledgeNode, AppError> {
        let title = title.trim();
        if title.is_empty() {
            return Err(AppError::Validation(
                "knowledge title must not be empty".to_owned(),
            ));
        }

        let content = content.trim();
        if content.is_empty() {
            return Err(AppError::Validation(
                "knowledge content must not be empty".to_owned(),
            ));
        }

        let knowledge_type =
            KnowledgeType::try_from(knowledge_type.trim()).map_err(AppError::Validation)?;
        let workspace = self.workspace_repository.ensure_default_workspace()?;

        self.knowledge_repository
            .insert_manual_node(&workspace.id, title, content, knowledge_type)
    }

    fn accept_knowledge_node(&self, knowledge_id: String) -> Result<KnowledgeNode, AppError> {
        let knowledge_id = validate_knowledge_id(&knowledge_id)?;
        let workspace = self.workspace_repository.ensure_default_workspace()?;
        self.knowledge_repository
            .accept_proposed_node(&workspace.id, knowledge_id)
    }

    fn archive_knowledge_node(&self, knowledge_id: String) -> Result<KnowledgeNode, AppError> {
        let knowledge_id = validate_knowledge_id(&knowledge_id)?;
        let workspace = self.workspace_repository.ensure_default_workspace()?;
        self.knowledge_repository
            .archive_proposed_node(&workspace.id, knowledge_id)
    }

    fn list_knowledge_nodes(
        &self,
        status: Option<String>,
        knowledge_type: Option<String>,
        query: Option<String>,
        limit: usize,
    ) -> Result<Vec<KnowledgeNode>, AppError> {
        let status = status
            .as_deref()
            .map(str::trim)
            .map(KnowledgeStatus::try_from)
            .transpose()
            .map_err(AppError::Validation)?;
        let knowledge_type = knowledge_type
            .as_deref()
            .map(str::trim)
            .map(KnowledgeType::try_from)
            .transpose()
            .map_err(AppError::Validation)?;
        let query = query
            .as_deref()
            .map(str::trim)
            .filter(|query| !query.is_empty());
        let workspace = self.workspace_repository.ensure_default_workspace()?;
        self.knowledge_repository
            .list_nodes(&workspace.id, status, knowledge_type, query, limit)
    }
}

fn validate_knowledge_id(knowledge_id: &str) -> Result<&str, AppError> {
    let knowledge_id = knowledge_id.trim();
    if knowledge_id.is_empty() {
        return Err(AppError::Validation(
            "knowledge_id must not be empty".to_owned(),
        ));
    }

    Ok(knowledge_id)
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{DefaultKnowledgeService, KnowledgeService};
    use crate::{
        domain::{ports::WorkspaceRepository, KnowledgeStatus, KnowledgeType},
        error::AppError,
        infrastructure::database::{
            repositories::{SqliteKnowledgeRepository, SqliteWorkspaceRepository},
            Database,
        },
    };

    #[test]
    fn creates_an_accepted_node_in_the_default_workspace() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let service = DefaultKnowledgeService::new(&workspace_repository, &knowledge_repository);

        let node = service
            .create_knowledge_node(
                "  Local First  ".to_owned(),
                "  Data stays local.  ".to_owned(),
                "concept".to_owned(),
            )
            .expect("create knowledge node");
        let default_workspace = workspace_repository
            .find_default_workspace()
            .expect("find default workspace")
            .expect("default workspace should exist");
        let nodes = service
            .list_knowledge_nodes(None, None, None, 50)
            .expect("list knowledge nodes");

        assert_eq!(node.workspace_id, default_workspace.id);
        assert_eq!(node.title, "Local First");
        assert_eq!(node.content, "Data stays local.");
        assert_eq!(node.knowledge_type, KnowledgeType::Concept);
        assert_eq!(node.status, KnowledgeStatus::Accepted);
        assert_eq!(nodes, vec![node]);
    }

    #[test]
    fn rejects_empty_title_content_and_unknown_type() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let service = DefaultKnowledgeService::new(&workspace_repository, &knowledge_repository);

        for result in [
            service.create_knowledge_node(
                " \n".to_owned(),
                "Content".to_owned(),
                "concept".to_owned(),
            ),
            service.create_knowledge_node(
                "Title".to_owned(),
                "\t".to_owned(),
                "concept".to_owned(),
            ),
            service.create_knowledge_node(
                "Title".to_owned(),
                "Content".to_owned(),
                "note".to_owned(),
            ),
        ] {
            assert!(matches!(result, Err(AppError::Validation(_))));
        }
    }

    #[test]
    fn rejects_empty_ids_for_knowledge_review_operations() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let service = DefaultKnowledgeService::new(&workspace_repository, &knowledge_repository);

        assert!(matches!(
            service.accept_knowledge_node(" \n".to_owned()),
            Err(AppError::Validation(_))
        ));
        assert!(matches!(
            service.archive_knowledge_node("\t".to_owned()),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn parses_optional_knowledge_filters_and_rejects_unknown_values() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let service = DefaultKnowledgeService::new(&workspace_repository, &knowledge_repository);

        service
            .list_knowledge_nodes(
                Some("accepted".to_owned()),
                Some("concept".to_owned()),
                None,
                50,
            )
            .expect("parse supported filters");

        assert!(matches!(
            service.list_knowledge_nodes(Some("reviewed".to_owned()), None, None, 50),
            Err(AppError::Validation(_))
        ));
        assert!(matches!(
            service.list_knowledge_nodes(None, Some("note".to_owned()), None, 50),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn trims_search_queries_and_treats_blank_queries_as_absent() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let service = DefaultKnowledgeService::new(&workspace_repository, &knowledge_repository);
        service
            .create_knowledge_node(
                "Local First".to_owned(),
                "Data stays local.".to_owned(),
                "concept".to_owned(),
            )
            .expect("create matching node");
        service
            .create_knowledge_node(
                "SQLite".to_owned(),
                "Embedded database.".to_owned(),
                "tool".to_owned(),
            )
            .expect("create non-matching node");

        let searched = service
            .list_knowledge_nodes(None, None, Some("  LOCAL FIRST  ".to_owned()), 50)
            .expect("search trimmed query");
        let blank = service
            .list_knowledge_nodes(None, None, Some(" \n\t ".to_owned()), 50)
            .expect("treat blank query as absent");
        let unfiltered = service
            .list_knowledge_nodes(None, None, None, 50)
            .expect("list without query");

        assert_eq!(searched.len(), 1);
        assert_eq!(searched[0].title, "Local First");
        assert_eq!(blank, unfiltered);
    }

    fn test_database() -> Database {
        Database::from_connection(Connection::open_in_memory().expect("open in-memory database"))
            .expect("initialize test database")
    }
}
