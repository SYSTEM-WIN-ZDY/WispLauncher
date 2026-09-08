ALTER TABLE settings ADD COLUMN mojang_auth_source TEXT NOT NULL DEFAULT 'auto' CHECK (
	mojang_auth_source IN ('auto', 'official_only', 'mirror_preferred', 'official_preferred')
);
