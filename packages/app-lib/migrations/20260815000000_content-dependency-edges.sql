ALTER TABLE instance_content_entries
	ADD COLUMN auto_dependency INTEGER NOT NULL DEFAULT 0
	CHECK (auto_dependency IN (0, 1));

CREATE INDEX instance_content_entries_auto_dependency
	ON instance_content_entries(content_set_id, auto_dependency);

CREATE TABLE instance_content_dependencies (
	id TEXT NOT NULL,
	content_set_id TEXT NOT NULL,
	parent_entry_id TEXT NOT NULL,
	child_entry_id TEXT NOT NULL,
	provider TEXT NOT NULL,
	dependency_kind TEXT NOT NULL,
	parent_project_id TEXT NOT NULL,
	parent_release_id TEXT NOT NULL,
	child_project_id TEXT NOT NULL,
	child_release_id TEXT NOT NULL,
	created_at INTEGER NOT NULL,
	modified_at INTEGER NOT NULL,

	PRIMARY KEY (id),
	UNIQUE (
		content_set_id,
		parent_entry_id,
		child_entry_id,
		dependency_kind
	),
	FOREIGN KEY (content_set_id)
		REFERENCES instance_content_sets(id)
		ON DELETE CASCADE,
	FOREIGN KEY (parent_entry_id)
		REFERENCES instance_content_entries(id)
		ON DELETE CASCADE,
	FOREIGN KEY (child_entry_id)
		REFERENCES instance_content_entries(id)
		ON DELETE CASCADE,
	CHECK (provider IN ('modrinth', 'curseforge')),
	CHECK (dependency_kind IN ('required', 'include'))
);

CREATE INDEX instance_content_dependencies_child
	ON instance_content_dependencies(content_set_id, child_entry_id);
CREATE INDEX instance_content_dependencies_parent
	ON instance_content_dependencies(content_set_id, parent_entry_id);
CREATE INDEX instance_content_dependencies_child_project
	ON instance_content_dependencies(provider, child_project_id);
