CREATE TABLE offline_minecraft_skins (
	minecraft_user_uuid TEXT NOT NULL,
	texture_key TEXT NOT NULL,
	variant TEXT NOT NULL CHECK (variant IN ('CLASSIC', 'SLIM', 'UNKNOWN')),

	PRIMARY KEY (minecraft_user_uuid)
);

-- Keep offline skin selections when an account is temporarily removed, matching
-- the saved-skin tables, while still rejecting selections for unknown users.
CREATE TRIGGER offline_minecraft_skins_user_uuid_insert_check
	BEFORE INSERT ON offline_minecraft_skins FOR EACH ROW
	BEGIN
		SELECT CASE WHEN NOT EXISTS (
			SELECT 1 FROM minecraft_users WHERE uuid = NEW.minecraft_user_uuid
		) THEN RAISE(ABORT, 'Cannot add an offline skin for an unknown Minecraft user UUID') END;
	END;

CREATE TRIGGER offline_minecraft_skins_user_uuid_update_check
	BEFORE UPDATE ON offline_minecraft_skins FOR EACH ROW
	BEGIN
		SELECT CASE WHEN NOT EXISTS (
			SELECT 1 FROM minecraft_users WHERE uuid = NEW.minecraft_user_uuid
		) THEN RAISE(ABORT, 'Cannot change an offline skin to refer to an unknown Minecraft user UUID') END;
	END;

CREATE TRIGGER offline_minecraft_skins_user_uuid_update_cascade
	AFTER UPDATE OF uuid ON minecraft_users FOR EACH ROW
	BEGIN
		UPDATE offline_minecraft_skins
		SET minecraft_user_uuid = NEW.uuid
		WHERE minecraft_user_uuid = OLD.uuid;
	END;
