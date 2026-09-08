CREATE TABLE IF NOT EXISTS java_versions_new (
    major_version INTEGER NOT NULL,
    full_version TEXT NOT NULL,
    architecture TEXT NOT NULL,
    path TEXT NOT NULL PRIMARY KEY,
    distribution TEXT
);

INSERT OR REPLACE INTO java_versions_new (major_version, full_version, architecture, path, distribution)
    SELECT major_version, full_version, architecture, path, NULL FROM java_versions
    GROUP BY path;

DROP TABLE IF EXISTS java_versions;
ALTER TABLE java_versions_new RENAME TO java_versions;

CREATE INDEX IF NOT EXISTS idx_java_versions_major_version ON java_versions(major_version);
