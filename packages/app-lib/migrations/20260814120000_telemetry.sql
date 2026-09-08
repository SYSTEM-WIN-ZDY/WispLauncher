ALTER TABLE settings
	ADD COLUMN telemetry_consent_version INTEGER NOT NULL DEFAULT 0;

CREATE TABLE telemetry_identity (
	id INTEGER NOT NULL CHECK (id = 0),
	installation_id TEXT NOT NULL,
	last_heartbeat_day TEXT NULL,
	PRIMARY KEY (id)
);

CREATE TABLE telemetry_outbox (
	event_id TEXT NOT NULL,
	event_type TEXT NOT NULL CHECK (event_type IN ('heartbeat', 'error')),
	payload JSONB NOT NULL,
	created_at INTEGER NOT NULL,
	next_attempt_at INTEGER NOT NULL,
	attempts INTEGER NOT NULL DEFAULT 0,
	size_bytes INTEGER NOT NULL,
	dedupe_key TEXT NOT NULL,
	occurrence_count INTEGER NOT NULL DEFAULT 1,
	PRIMARY KEY (event_id),
	UNIQUE (dedupe_key)
);

CREATE INDEX telemetry_outbox_retry
	ON telemetry_outbox (next_attempt_at, created_at);

CREATE TABLE telemetry_error_daily (
	day TEXT NOT NULL,
	fingerprint TEXT NOT NULL,
	PRIMARY KEY (day, fingerprint)
);
