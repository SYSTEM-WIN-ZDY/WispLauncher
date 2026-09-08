ALTER TABLE settings ADD COLUMN custom_background_path TEXT NULL;
ALTER TABLE settings ADD COLUMN custom_background_blur INTEGER NOT NULL DEFAULT 12 CHECK (custom_background_blur BETWEEN 0 AND 40);
ALTER TABLE settings ADD COLUMN custom_background_opacity INTEGER NOT NULL DEFAULT 65 CHECK (custom_background_opacity BETWEEN 10 AND 100);
