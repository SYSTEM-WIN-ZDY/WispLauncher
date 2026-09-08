ALTER TABLE settings ADD COLUMN onboarding_version INTEGER NOT NULL DEFAULT 0;
ALTER TABLE settings ADD COLUMN onboarding_instance_tour_completed INTEGER NOT NULL DEFAULT TRUE;

UPDATE settings
SET onboarding_instance_tour_completed = CASE WHEN onboarded = 1 THEN TRUE ELSE FALSE END;
