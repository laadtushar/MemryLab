-- PII flags for memory facts
CREATE TABLE IF NOT EXISTS pii_scan_results (
    fact_id TEXT PRIMARY KEY,
    pii_types TEXT NOT NULL DEFAULT '[]',
    scanned_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (fact_id) REFERENCES memory_facts(id)
);
