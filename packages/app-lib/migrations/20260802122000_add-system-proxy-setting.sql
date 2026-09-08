ALTER TABLE settings
	ADD COLUMN use_system_proxy INTEGER NOT NULL DEFAULT FALSE
	CHECK (use_system_proxy IN (0, 1));
