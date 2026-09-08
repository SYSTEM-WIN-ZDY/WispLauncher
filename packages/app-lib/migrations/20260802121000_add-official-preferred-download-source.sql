CREATE TABLE settings_with_official_preferred (
	id INTEGER NOT NULL CHECK (id = 0),
	max_concurrent_downloads INTEGER NOT NULL DEFAULT 10,
	max_concurrent_writes INTEGER NOT NULL DEFAULT 10,
	theme TEXT NOT NULL DEFAULT 'dark',
	default_page TEXT NOT NULL DEFAULT 'home',
	collapsed_navigation INTEGER NOT NULL DEFAULT TRUE,
	advanced_rendering INTEGER NOT NULL DEFAULT TRUE,
	native_decorations INTEGER NOT NULL DEFAULT FALSE,
	telemetry INTEGER NOT NULL DEFAULT FALSE,
	discord_rpc INTEGER NOT NULL DEFAULT TRUE,
	developer_mode INTEGER NOT NULL DEFAULT FALSE,
	onboarded INTEGER NOT NULL DEFAULT FALSE,
	extra_launch_args JSONB NOT NULL,
	custom_env_vars JSONB NOT NULL,
	mc_memory_max INTEGER NOT NULL DEFAULT 2048,
	mc_force_fullscreen INTEGER NOT NULL DEFAULT FALSE,
	mc_game_resolution_x INTEGER NOT NULL DEFAULT 854,
	mc_game_resolution_y INTEGER NOT NULL DEFAULT 480,
	hide_on_process_start INTEGER NOT NULL DEFAULT FALSE,
	hook_pre_launch TEXT NULL,
	hook_wrapper TEXT NULL,
	hook_post_exit TEXT NULL,
	custom_dir TEXT NULL,
	prev_custom_dir TEXT NULL,
	migrated INTEGER NOT NULL DEFAULT FALSE,
	personalized_ads INTEGER NOT NULL DEFAULT TRUE,
	toggle_sidebar INTEGER NOT NULL DEFAULT FALSE,
	feature_flags JSONB NOT NULL DEFAULT '{}',
	hide_nametag_skins_page INTEGER NOT NULL DEFAULT 0 CHECK (hide_nametag_skins_page IN (0, 1)),
	skipped_update TEXT NULL,
	pending_update_toast_for_version TEXT NULL,
	auto_download_updates INT NULL,
	version INTEGER NOT NULL DEFAULT 1,
	locale TEXT NOT NULL DEFAULT '',
	accent_color TEXT NOT NULL DEFAULT 'blue',
	custom_background_path TEXT NULL,
	custom_background_blur INTEGER NOT NULL DEFAULT 12 CHECK (custom_background_blur BETWEEN 0 AND 40),
	custom_background_opacity INTEGER NOT NULL DEFAULT 65 CHECK (custom_background_opacity BETWEEN 10 AND 100),
	use_minecraft_mirror INTEGER NOT NULL DEFAULT FALSE,
	use_modrinth_mirror INTEGER NOT NULL DEFAULT FALSE,
	use_curseforge_mirror INTEGER NOT NULL DEFAULT TRUE,
	auto_concurrent_downloads INTEGER NOT NULL DEFAULT TRUE,
	minecraft_metadata_source TEXT NOT NULL DEFAULT 'auto' CHECK (
		minecraft_metadata_source IN ('auto', 'official_only', 'mirror_preferred', 'official_preferred')
	),
	minecraft_file_source TEXT NOT NULL DEFAULT 'auto' CHECK (
		minecraft_file_source IN ('auto', 'official_only', 'mirror_preferred', 'official_preferred')
	),
	modrinth_source TEXT NOT NULL DEFAULT 'auto' CHECK (
		modrinth_source IN ('auto', 'official_only', 'mirror_preferred', 'official_preferred')
	),
	curseforge_source TEXT NOT NULL DEFAULT 'auto' CHECK (
		curseforge_source IN ('auto', 'official_only', 'mirror_preferred', 'official_preferred')
	),
	onboarding_version INTEGER NOT NULL DEFAULT 0,
	onboarding_instance_tour_completed INTEGER NOT NULL DEFAULT TRUE,
	sidebar_instance_count INTEGER NOT NULL DEFAULT 0,
	mc_memory_auto INTEGER NOT NULL DEFAULT FALSE,
	mc_memory_optimize INTEGER NOT NULL DEFAULT FALSE,
	auto_set_java_high_performance_mode INTEGER NOT NULL DEFAULT TRUE,
	transparent_background INTEGER NOT NULL DEFAULT FALSE,
	transparent_background_opacity INTEGER NOT NULL DEFAULT 55,
	transparent_background_blur INTEGER NOT NULL DEFAULT FALSE,
	home_layout TEXT NOT NULL DEFAULT 'standard' CHECK (home_layout IN ('standard', 'minimal')),
	minimal_home_instance_id TEXT NULL REFERENCES instances(id) ON DELETE SET NULL,
	auto_hide_downloads_button INTEGER NOT NULL DEFAULT FALSE,
	PRIMARY KEY (id)
);

INSERT INTO settings_with_official_preferred
SELECT * FROM settings;

DROP TABLE settings;
ALTER TABLE settings_with_official_preferred RENAME TO settings;
