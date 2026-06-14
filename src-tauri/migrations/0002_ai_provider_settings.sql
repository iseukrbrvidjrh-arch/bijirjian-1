CREATE TABLE ai_provider_settings (
    id TEXT PRIMARY KEY NOT NULL CHECK (id = 'default'),
    provider_type TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
