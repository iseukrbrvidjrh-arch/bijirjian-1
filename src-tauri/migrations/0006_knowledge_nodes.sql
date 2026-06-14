CREATE TABLE knowledge_nodes (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL,
    title TEXT NOT NULL CHECK (
        length(trim(title, char(9) || char(10) || char(11) || char(12) || char(13) || ' ')) > 0
    ),
    content TEXT NOT NULL CHECK (
        length(trim(content, char(9) || char(10) || char(11) || char(12) || char(13) || ' ')) > 0
    ),
    knowledge_type TEXT NOT NULL CHECK (
        knowledge_type IN (
            'concept',
            'tool',
            'project',
            'question',
            'solution',
            'insight',
            'resource',
            'person'
        )
    ),
    status TEXT NOT NULL DEFAULT 'accepted' CHECK (
        status IN ('proposed', 'accepted', 'archived')
    ),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    archived_at TEXT,
    FOREIGN KEY (workspace_id) REFERENCES workspaces (id) ON DELETE RESTRICT
);

CREATE INDEX idx_knowledge_nodes_workspace_created
    ON knowledge_nodes (workspace_id, created_at DESC);
