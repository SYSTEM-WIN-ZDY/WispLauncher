CREATE TABLE IF NOT EXISTS google_translate_ip_cache (
	ip TEXT NOT NULL PRIMARY KEY,
	latency_ms INTEGER NOT NULL,
	created_at INTEGER NOT NULL
);
