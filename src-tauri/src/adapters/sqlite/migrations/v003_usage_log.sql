CREATE TABLE IF NOT EXISTS llm_usage_log (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    prompt_tokens INTEGER DEFAULT 0,
    completion_tokens INTEGER DEFAULT 0,
    purpose TEXT DEFAULT '',
    duration_ms INTEGER DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_usage_log_timestamp ON llm_usage_log(timestamp);
