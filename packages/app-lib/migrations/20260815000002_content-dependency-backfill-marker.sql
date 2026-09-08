-- Track which content entries already had their CurseForge dependency
-- relations resolved during backfill, so entries without any relations are
-- not refetched from the API on every snapshot.

ALTER TABLE instance_content_entries
	ADD COLUMN dependency_backfilled INTEGER NOT NULL DEFAULT 0
	CHECK (dependency_backfilled IN (0, 1));

CREATE INDEX instance_content_entries_dependency_backfilled
	ON instance_content_entries(content_set_id, dependency_backfilled);
