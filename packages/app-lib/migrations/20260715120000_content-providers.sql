ALTER TABLE instance_links
	ADD COLUMN curseforge_project_id INTEGER NULL;

ALTER TABLE instance_links
	ADD COLUMN curseforge_file_id INTEGER NULL;

CREATE INDEX instance_links_curseforge_project_id
	ON instance_links(curseforge_project_id);

CREATE TABLE instance_content_provider_refs (
	content_entry_id TEXT NOT NULL,
	provider TEXT NOT NULL,
	project_id TEXT NOT NULL,
	version_id TEXT NULL,
	primary_ref INTEGER NOT NULL DEFAULT 0,

	PRIMARY KEY (content_entry_id, provider),
	FOREIGN KEY (content_entry_id)
		REFERENCES instance_content_entries(id)
		ON DELETE CASCADE,
	CHECK (provider IN ('modrinth', 'curseforge')),
	CHECK (primary_ref IN (0, 1))
);

CREATE INDEX instance_content_provider_refs_project
	ON instance_content_provider_refs(provider, project_id);

CREATE INDEX instance_content_provider_refs_version
	ON instance_content_provider_refs(provider, version_id);

CREATE UNIQUE INDEX instance_content_provider_refs_primary
	ON instance_content_provider_refs(content_entry_id)
	WHERE primary_ref = 1;

INSERT OR IGNORE INTO instance_content_provider_refs (
	content_entry_id,
	provider,
	project_id,
	version_id,
	primary_ref
)
SELECT
	id,
	'modrinth',
	project_id,
	version_id,
	1
FROM instance_content_entries
WHERE project_id IS NOT NULL;
