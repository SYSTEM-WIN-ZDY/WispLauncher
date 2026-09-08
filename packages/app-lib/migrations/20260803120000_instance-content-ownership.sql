ALTER TABLE instance_content_sets
	ADD COLUMN revision INTEGER NOT NULL DEFAULT 0;

ALTER TABLE instance_content_entries
	ADD COLUMN ownership_kind TEXT NOT NULL DEFAULT 'user_added'
	CHECK (ownership_kind IN ('pack_managed', 'user_added'));

UPDATE instance_content_entries AS entry
SET ownership_kind = 'pack_managed'
WHERE EXISTS (
	SELECT 1
	FROM instance_links link
	WHERE link.instance_id = entry.instance_id
		AND (
			(link.link_kind IN ('modrinth_modpack', 'server_project_modpack')
				AND entry.source_kind = 'modrinth_modpack')
			OR (link.link_kind = 'curseforge_modpack'
				AND entry.source_kind = 'curseforge')
			OR (link.link_kind = 'imported_modpack'
				AND entry.source_kind IN (
					'imported_modpack',
					'modrinth_modpack',
					'curseforge'
				))
		)
);

CREATE INDEX instance_content_entries_ownership_kind
	ON instance_content_entries(content_set_id, ownership_kind);

CREATE TABLE instance_pack_members (
	id TEXT NOT NULL,
	content_set_id TEXT NOT NULL,
	content_entry_id TEXT NULL,
	member_key TEXT NOT NULL,
	project_type TEXT NOT NULL,
	expected_relative_path TEXT NOT NULL,
	provider TEXT NULL,
	provider_project_id TEXT NULL,
	provider_release_id TEXT NULL,
	required INTEGER NOT NULL DEFAULT 1,
	expected_sha1 TEXT NULL,
	expected_size INTEGER NULL,
	expected_fingerprint INTEGER NULL,
	materialization_state TEXT NOT NULL DEFAULT 'present',
	override_kind TEXT NOT NULL DEFAULT 'none',
	reconciled INTEGER NOT NULL DEFAULT 1,
	created_at INTEGER NOT NULL,
	modified_at INTEGER NOT NULL,

	PRIMARY KEY (id),
	UNIQUE (content_set_id, member_key),
	FOREIGN KEY (content_set_id)
		REFERENCES instance_content_sets(id)
		ON DELETE CASCADE,
	FOREIGN KEY (content_entry_id)
		REFERENCES instance_content_entries(id)
		ON DELETE SET NULL,
	CHECK (provider IS NULL OR provider IN ('modrinth', 'curseforge')),
	CHECK (required IN (0, 1)),
	CHECK (materialization_state IN (
		'present',
		'pending_manual',
		'missing',
		'removed'
	)),
	CHECK (override_kind IN ('none', 'disabled', 'removed', 'version')),
	CHECK (reconciled IN (0, 1))
);

CREATE INDEX instance_pack_members_content_set
	ON instance_pack_members(content_set_id);
CREATE INDEX instance_pack_members_content_entry
	ON instance_pack_members(content_entry_id);
CREATE INDEX instance_pack_members_provider
	ON instance_pack_members(provider, provider_project_id, provider_release_id);
CREATE INDEX instance_pack_members_state
	ON instance_pack_members(content_set_id, materialization_state);

INSERT INTO instance_pack_members (
	id,
	content_set_id,
	content_entry_id,
	member_key,
	project_type,
	expected_relative_path,
	provider,
	provider_project_id,
	provider_release_id,
	required,
	expected_sha1,
	expected_size,
	expected_fingerprint,
	materialization_state,
	override_kind,
	reconciled,
	created_at,
	modified_at
)
SELECT
	'pack-member:' || entry.id,
	entry.content_set_id,
	entry.id,
	CASE
		WHEN origin.provider IS NOT NULL THEN
			origin.provider || ':' || origin.provider_project_id || ':' || entry.project_type
		ELSE 'path:' || lower(replace(file.relative_path, '\\', '/'))
	END,
	entry.project_type,
	file.relative_path,
	origin.provider,
	origin.provider_project_id,
	origin.provider_release_id,
	CASE
		WHEN entry.server_requirement = 'optional'
			AND entry.client_requirement = 'optional'
			THEN 0
		ELSE 1
	END,
	file.sha1,
	file.size,
	NULL,
	CASE WHEN file.missing = 1 THEN 'missing' ELSE 'present' END,
	CASE
		WHEN entry.enabled = 0 OR file.enabled = 0 THEN 'disabled'
		ELSE 'none'
	END,
	CASE
		WHEN link.link_kind = 'curseforge_modpack' THEN 0
		ELSE 1
	END,
	entry.added_at,
	entry.modified_at
FROM instance_content_entries entry
INNER JOIN instance_files file ON file.id = entry.file_id
INNER JOIN instance_links link ON link.instance_id = entry.instance_id
LEFT JOIN instance_content_provider_refs origin
	ON origin.content_entry_id = entry.id
	AND origin.is_origin = 1
WHERE entry.ownership_kind = 'pack_managed';

CREATE TABLE instance_pending_manual_downloads (
	id TEXT NOT NULL,
	instance_id TEXT NOT NULL,
	pack_member_id TEXT NULL,
	content_entry_id TEXT NULL,
	operation_kind TEXT NOT NULL,
	operation_target_id TEXT NULL,
	project_type TEXT NOT NULL,
	provider TEXT NOT NULL,
	provider_project_id TEXT NOT NULL,
	provider_release_id TEXT NOT NULL,
	file_name TEXT NOT NULL,
	website_url TEXT NULL,
	target_relative_path TEXT NOT NULL,
	expected_sha1 TEXT NULL,
	expected_size INTEGER NULL,
	expected_fingerprint INTEGER NULL,
	state TEXT NOT NULL DEFAULT 'waiting',
	context JSONB NOT NULL DEFAULT '{}',
	created_at INTEGER NOT NULL,
	modified_at INTEGER NOT NULL,

	PRIMARY KEY (id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE,
	FOREIGN KEY (pack_member_id)
		REFERENCES instance_pack_members(id)
		ON DELETE CASCADE,
	FOREIGN KEY (content_entry_id)
		REFERENCES instance_content_entries(id)
		ON DELETE SET NULL,
	CHECK (provider IN ('modrinth', 'curseforge')),
	CHECK (operation_kind IN (
		'pack_install',
		'pack_update',
		'content_install',
		'content_update'
	)),
	CHECK (state IN ('waiting', 'matched', 'imported', 'error', 'cancelled'))
);

CREATE UNIQUE INDEX instance_pending_manual_download_identity
	ON instance_pending_manual_downloads(
		instance_id,
		operation_kind,
		provider,
		provider_project_id,
		provider_release_id
	)
	WHERE state IN ('waiting', 'matched');
CREATE INDEX instance_pending_manual_downloads_instance_state
	ON instance_pending_manual_downloads(instance_id, state);
