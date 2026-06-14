CREATE TABLE ai_runs (
    id TEXT PRIMARY KEY NOT NULL,
    source_id TEXT NOT NULL,
    prompt_version_id TEXT,
    provider_type TEXT,
    model TEXT,
    status TEXT NOT NULL CHECK (status IN ('succeeded', 'failed')),
    output_text TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL,
    completed_at TEXT NOT NULL,
    FOREIGN KEY (source_id) REFERENCES sources (id) ON DELETE RESTRICT,
    FOREIGN KEY (prompt_version_id) REFERENCES prompt_versions (id) ON DELETE RESTRICT,
    CHECK (
        (provider_type IS NULL AND model IS NULL)
        OR (provider_type IS NOT NULL AND model IS NOT NULL)
    ),
    CHECK (
        (
            status = 'succeeded'
            AND output_text IS NOT NULL
            AND length(trim(output_text)) > 0
            AND error_message IS NULL
        )
        OR (
            status = 'failed'
            AND output_text IS NULL
            AND error_message IS NOT NULL
            AND length(trim(error_message)) > 0
        )
    )
);

CREATE INDEX idx_ai_runs_source_created
    ON ai_runs (source_id, created_at DESC);
