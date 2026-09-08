ALTER TABLE install_jobs ADD COLUMN provider TEXT NULL;
ALTER TABLE install_jobs ADD COLUMN retry_of TEXT NULL;
ALTER TABLE install_jobs ADD COLUMN files_total INTEGER NULL;
ALTER TABLE install_jobs ADD COLUMN files_completed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE install_jobs ADD COLUMN bytes_total INTEGER NULL;
ALTER TABLE install_jobs ADD COLUMN bytes_downloaded INTEGER NOT NULL DEFAULT 0;

CREATE TABLE install_job_items (
	id TEXT NOT NULL,
	job_id TEXT NOT NULL,
	name TEXT NOT NULL,
	project_id TEXT NULL,
	version_id TEXT NULL,
	item_type TEXT NULL,
	status TEXT NOT NULL,
	url TEXT NULL,
	bytes_total INTEGER NULL,
	bytes_downloaded INTEGER NOT NULL DEFAULT 0,
	error TEXT NULL,
	manual_url TEXT NULL,
	created INTEGER NOT NULL,
	modified INTEGER NOT NULL,
	finished INTEGER NULL,
	PRIMARY KEY (job_id, id),
	FOREIGN KEY (job_id) REFERENCES install_jobs(id) ON DELETE CASCADE
);

CREATE INDEX install_jobs_provider ON install_jobs(provider);
CREATE INDEX install_job_items_job_id ON install_job_items(job_id);
CREATE INDEX install_job_items_status ON install_job_items(status);
