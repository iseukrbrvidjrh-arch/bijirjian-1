CREATE TABLE export_records (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL,
    knowledge_node_id TEXT NOT NULL,
    export_path TEXT,
    status TEXT NOT NULL CHECK (
        status IN ('succeeded', 'failed')
    ),
    error_message TEXT,
    created_at TEXT NOT NULL,
    CHECK (
        (
            status = 'succeeded'
            AND export_path IS NOT NULL
            AND length(trim(export_path, char(9) || char(10) || char(11) || char(12) || char(13) || ' ')) > 0
            AND error_message IS NULL
        )
        OR
        (
            status = 'failed'
            AND error_message IS NOT NULL
            AND length(trim(error_message, char(9) || char(10) || char(11) || char(12) || char(13) || ' ')) > 0
            AND (
                export_path IS NULL
                OR length(trim(export_path, char(9) || char(10) || char(11) || char(12) || char(13) || ' ')) > 0
            )
        )
    ),
    FOREIGN KEY (workspace_id) REFERENCES workspaces (id) ON DELETE RESTRICT,
    FOREIGN KEY (knowledge_node_id) REFERENCES knowledge_nodes (id) ON DELETE RESTRICT
);

CREATE INDEX idx_export_records_workspace_knowledge_created
    ON export_records (workspace_id, knowledge_node_id, created_at DESC);
