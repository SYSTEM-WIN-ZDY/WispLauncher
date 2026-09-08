ALTER TABLE ai_settings
	ADD COLUMN legacy_openai_credential_cleanup INTEGER NOT NULL DEFAULT FALSE;

CREATE TEMP TABLE legacy_openai_ai_config (
	provider_id TEXT NOT NULL PRIMARY KEY,
	model_id TEXT NOT NULL
);

INSERT INTO legacy_openai_ai_config (provider_id, model_id)
SELECT
	config.provider_id,
	translation.openai_model
FROM translation_settings AS translation
INNER JOIN ai_provider_configs AS config
	ON config.provider_id = 'openai'
	AND config.custom_name = 'OpenAI Compatible'
	AND config.protocol = 'openai'
	AND config.enabled = TRUE
	AND config.endpoint = translation.openai_base_url
	AND json(config.settings) = json('{}')
INNER JOIN ai_provider_models AS model
	ON model.provider_id = config.provider_id
	AND model.model_id = translation.openai_model
	AND model.display_name = translation.openai_model
	AND model.enabled = TRUE
	AND model.source = 'custom'
WHERE translation.id = 0;

UPDATE ai_settings
SET legacy_openai_credential_cleanup = TRUE
WHERE id = 0
	AND (
		EXISTS (SELECT 1 FROM legacy_openai_ai_config)
		OR EXISTS (
			SELECT 1
			FROM translation_settings
			WHERE id = 0 AND openai_api_key IS NOT NULL
		)
	);

DELETE FROM ai_provider_models
WHERE EXISTS (
	SELECT 1
	FROM legacy_openai_ai_config
	WHERE legacy_openai_ai_config.provider_id = ai_provider_models.provider_id
		AND legacy_openai_ai_config.model_id = ai_provider_models.model_id
);

DELETE FROM ai_provider_configs
WHERE EXISTS (
	SELECT 1
	FROM legacy_openai_ai_config
	WHERE legacy_openai_ai_config.provider_id = ai_provider_configs.provider_id
)
	AND NOT EXISTS (
		SELECT 1
		FROM ai_provider_models
		WHERE ai_provider_models.provider_id = ai_provider_configs.provider_id
	);

UPDATE translation_settings
SET provider = CASE
		WHEN provider = 'ai'
			AND ai_provider_id = 'openai'
			AND EXISTS (
				SELECT 1
				FROM legacy_openai_ai_config
				WHERE model_id = ai_model_id
			)
		THEN 'microsoft'
		ELSE provider
	END,
	ai_provider_id = CASE
		WHEN provider = 'ai'
			AND ai_provider_id = 'openai'
			AND EXISTS (
				SELECT 1
				FROM legacy_openai_ai_config
				WHERE model_id = ai_model_id
			)
		THEN ''
		ELSE ai_provider_id
	END,
	ai_model_id = CASE
		WHEN provider = 'ai'
			AND ai_provider_id = 'openai'
			AND EXISTS (
				SELECT 1
				FROM legacy_openai_ai_config
				WHERE model_id = ai_model_id
			)
		THEN ''
		ELSE ai_model_id
	END,
	openai_base_url = 'https://api.openai.com/v1',
	openai_model = 'gpt-4o-mini',
	openai_api_key = NULL,
	openai_system_prompt = CASE
		WHEN provider = 'ai'
			AND NOT EXISTS (
				SELECT 1
				FROM legacy_openai_ai_config
				WHERE provider_id = ai_provider_id
					AND model_id = ai_model_id
			)
		THEN openai_system_prompt
		ELSE ''
	END
WHERE id = 0;

DROP TABLE legacy_openai_ai_config;
