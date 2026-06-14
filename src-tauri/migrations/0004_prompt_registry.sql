CREATE TABLE prompts (
    id TEXT PRIMARY KEY NOT NULL,
    prompt_key TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    active_version_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (active_version_id) REFERENCES prompt_versions (id) ON DELETE RESTRICT
);

CREATE TABLE prompt_versions (
    id TEXT PRIMARY KEY NOT NULL,
    prompt_id TEXT NOT NULL,
    version INTEGER NOT NULL CHECK (version > 0),
    prompt_content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (prompt_id) REFERENCES prompts (id) ON DELETE RESTRICT,
    UNIQUE (prompt_id, version)
);

INSERT INTO prompts (
    id,
    prompt_key,
    name,
    description,
    active_version_id,
    created_at,
    updated_at
)
VALUES (
    'builtin-source-summary',
    'source_summary',
    'Source Summary',
    'Summarizes a captured source into a concise and faithful note.',
    NULL,
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
);

INSERT INTO prompt_versions (
    id,
    prompt_id,
    version,
    prompt_content,
    created_at
)
VALUES (
    'builtin-source-summary-v1',
    'builtin-source-summary',
    1,
    'Summarize the provided source faithfully and concisely. Preserve important facts, decisions, terms, and open questions. Do not invent information. Return plain text.',
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
);

UPDATE prompts
SET active_version_id = 'builtin-source-summary-v1',
    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
WHERE id = 'builtin-source-summary';
