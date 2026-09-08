CREATE TABLE instance_config_sync_state (
	instance_id TEXT NOT NULL PRIMARY KEY,
	config_updated_at INTEGER NOT NULL,
	generated_at INTEGER NULL,
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);
