CREATE UNIQUE INDEX java_versions_major_version_path
	ON java_versions(major_version, path);

CREATE TABLE java_default_versions (
	major_version INTEGER NOT NULL PRIMARY KEY,
	path TEXT NOT NULL,
	FOREIGN KEY (major_version, path)
		REFERENCES java_versions(major_version, path)
		ON DELETE CASCADE
		ON UPDATE CASCADE
);

INSERT INTO java_default_versions (major_version, path)
	SELECT major_version, MIN(path)
	FROM java_versions
	GROUP BY major_version;
