-- Cache extracted icons for unmatched Mod / ResourcePack files so the
-- content list can show real artwork without reopening archives on every
-- render. NULL = not attempted yet, '' = attempted with no icon found.
ALTER TABLE instance_files
	ADD COLUMN icon_path TEXT NULL;
