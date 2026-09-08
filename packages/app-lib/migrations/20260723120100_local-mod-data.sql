-- Add local_mod_data column to instance_files for storing manually
-- extracted mod metadata (fabric.mod.json / quilt.mod.json / mods.toml
-- / neoforge.mods.toml / mcmod.info) when Modrinth hash lookup fails.

ALTER TABLE instance_files
	ADD COLUMN local_mod_data TEXT NULL;
