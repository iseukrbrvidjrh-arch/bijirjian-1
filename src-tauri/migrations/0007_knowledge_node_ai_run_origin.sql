ALTER TABLE knowledge_nodes
ADD COLUMN ai_run_id TEXT
    REFERENCES ai_runs (id) ON DELETE RESTRICT;

CREATE UNIQUE INDEX idx_knowledge_nodes_unique_ai_run_origin
    ON knowledge_nodes (ai_run_id)
    WHERE ai_run_id IS NOT NULL;
