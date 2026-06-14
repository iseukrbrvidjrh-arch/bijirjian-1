CREATE TABLE workspaces (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    archived_at TEXT,
    UNIQUE (name)
);

CREATE TABLE sources (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL,
    source_type TEXT NOT NULL,
    raw_content TEXT NOT NULL,
    content_hash TEXT,
    metadata_json TEXT,
    inbox_status TEXT NOT NULL,
    captured_at TEXT NOT NULL,
    processed_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    FOREIGN KEY (workspace_id) REFERENCES workspaces (id) ON DELETE RESTRICT
);

CREATE INDEX idx_sources_workspace_inbox_captured
    ON sources (workspace_id, inbox_status, captured_at);

CREATE INDEX idx_sources_workspace_content_hash
    ON sources (workspace_id, content_hash);
