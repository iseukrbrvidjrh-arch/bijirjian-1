CREATE TABLE obsidian_settings (
    workspace_id TEXT PRIMARY KEY NOT NULL,
    vault_path TEXT NOT NULL CHECK (
        length(trim(vault_path, char(9) || char(10) || char(11) || char(12) || char(13) || ' ')) > 0
    ),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspaces (id) ON DELETE RESTRICT
);
