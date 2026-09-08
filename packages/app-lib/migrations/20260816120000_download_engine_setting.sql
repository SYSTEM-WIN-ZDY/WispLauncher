ALTER TABLE settings
	ADD COLUMN download_engine TEXT NOT NULL DEFAULT 'xmcl';

UPDATE settings
SET download_engine = CASE
	WHEN json_extract(feature_flags, '$.xmcl_download_engine') = 0 THEN 'legacy'
	ELSE 'xmcl'
END;
