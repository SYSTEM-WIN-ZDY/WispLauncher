CREATE TABLE ai_settings (
	id INTEGER NOT NULL CHECK (id = 0),
	enabled INTEGER NOT NULL DEFAULT TRUE,
	PRIMARY KEY (id)
);

INSERT INTO ai_settings (id) VALUES (0);

CREATE TABLE ai_provider_configs (
	provider_id TEXT NOT NULL PRIMARY KEY,
	custom_name TEXT NULL,
	protocol TEXT NOT NULL DEFAULT 'openai',
	enabled INTEGER NOT NULL DEFAULT FALSE,
	endpoint TEXT NOT NULL DEFAULT '',
	settings TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE ai_provider_models (
	provider_id TEXT NOT NULL,
	model_id TEXT NOT NULL,
	display_name TEXT NOT NULL DEFAULT '',
	enabled INTEGER NOT NULL DEFAULT TRUE,
	source TEXT NOT NULL DEFAULT 'custom',
	PRIMARY KEY (provider_id, model_id)
);

ALTER TABLE translation_settings ADD COLUMN ai_provider_id TEXT NOT NULL DEFAULT '';
ALTER TABLE translation_settings ADD COLUMN ai_model_id TEXT NOT NULL DEFAULT '';

INSERT INTO ai_provider_configs (
	provider_id,
	custom_name,
	protocol,
	enabled,
	endpoint
)
SELECT
	'openai',
	'OpenAI Compatible',
	'openai',
	TRUE,
	openai_base_url
FROM translation_settings
WHERE id = 0
	AND trim(openai_model) != ''
	AND (
		lower(trim(openai_base_url)) LIKE 'http://%'
		OR lower(trim(openai_base_url)) LIKE 'https://%'
	)
	AND (
		provider = 'openai-compatible'
		OR openai_api_key IS NOT NULL
		OR openai_base_url != 'https://api.openai.com/v1'
		OR openai_model != 'gpt-4o-mini'
		OR openai_system_prompt != ''
	);

INSERT INTO ai_provider_models (
	provider_id,
	model_id,
	display_name,
	enabled,
	source
)
SELECT
	'openai',
	openai_model,
	openai_model,
	TRUE,
	'custom'
FROM translation_settings
WHERE id = 0
	AND EXISTS (
		SELECT 1
		FROM ai_provider_configs
		WHERE provider_id = 'openai'
	);

UPDATE translation_settings
SET provider = 'ai',
	ai_provider_id = 'openai',
	ai_model_id = openai_model
WHERE id = 0
	AND provider = 'openai-compatible'
	AND trim(openai_model) != ''
	AND (
		lower(trim(openai_base_url)) LIKE 'http://%'
		OR lower(trim(openai_base_url)) LIKE 'https://%'
	);

UPDATE translation_settings
SET provider = 'microsoft'
WHERE id = 0 AND provider = 'openai-compatible';
