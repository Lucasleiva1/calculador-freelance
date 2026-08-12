PRAGMA foreign_keys = ON;

-- Referencias argentinas separadas de los benchmarks globales. Aunque la
-- publicación original expresa tarifas en USD, las bandas corresponden
-- explícitamente a profesionales de Argentina y se convierten con el FX BCRA.
INSERT OR IGNORE INTO market_sources (
  id,name,base_url,source_type,regions_json,supported_services_json,priority,enabled,
  usage_mode,acquisition_mode,cooldown_hours,notes,is_system_source,system_key,
  default_data_json,purpose,data_contribution,app_benefit,participates_in_suggestions,
  automation_status,current_status,adapter_key,business_source_type,market_country,
  source_currency,source_updated_at,classification_origin,created_at,updated_at
) VALUES (
  'source-prolatam-video-ar','ProLatamWork · video Argentina 2026',
  'https://prolatamwork.com/blog/cuanto-cobra-editor-video-freelance-latinoamerica-2026',
  'rate_benchmark','["AR"]','["video-editing"]',12,1,'market_price','auto_http',24,
  'Bandas horarias publicadas para editores de video de Argentina por experiencia.',
  1,'prolatam-video-ar',
  json_object(
    'name','ProLatamWork · video Argentina 2026',
    'baseUrl','https://prolatamwork.com/blog/cuanto-cobra-editor-video-freelance-latinoamerica-2026',
    'sourceType','rate_benchmark','regionsJson','["AR"]',
    'supportedServicesJson','["video-editing"]','priority',12,'enabled',1,
    'usageMode','market_price','acquisitionMode','auto_http','cooldownHours',24,
    'purpose','Bandas horarias para editores freelance de Argentina por experiencia.',
    'dataContribution','Rangos junior, intermedio y senior en USD/hora con país y fecha.',
    'appBenefit','Calcula el precio de mercado argentino sin mezclarlo con el mercado global.',
    'participatesInSuggestions',1,'automationStatus','APPROVED','adapterKey','prolatam',
    'currentStatus','READY','lastError',NULL,'businessSourceType','market',
    'marketCountry','Argentina','sourceCurrency','USD','sourceUpdatedAt','2026-05-01'
  ),
  'Bandas horarias para editores freelance de Argentina por experiencia.',
  'Rangos junior, intermedio y senior en USD/hora con país y fecha.',
  'Calcula el precio de mercado argentino sin mezclarlo con el mercado global.',
  1,'APPROVED','READY','prolatam','market','Argentina','USD','2026-05-01','automatic',
  strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
);

INSERT OR IGNORE INTO market_sources (
  id,name,base_url,source_type,regions_json,supported_services_json,priority,enabled,
  usage_mode,acquisition_mode,cooldown_hours,notes,is_system_source,system_key,
  default_data_json,purpose,data_contribution,app_benefit,participates_in_suggestions,
  automation_status,current_status,adapter_key,business_source_type,market_country,
  source_currency,source_updated_at,classification_origin,created_at,updated_at
) VALUES (
  'source-prolatam-programming-ar','ProLatamWork · desarrollo Argentina 2026',
  'https://prolatamwork.com/blog/tarifas-desarrolladores-latam-2026',
  'rate_benchmark','["AR"]','["programming"]',12,1,'market_price','auto_http',24,
  'Bandas horarias publicadas para desarrolladores de Argentina por experiencia.',
  1,'prolatam-programming-ar',
  json_object(
    'name','ProLatamWork · desarrollo Argentina 2026',
    'baseUrl','https://prolatamwork.com/blog/tarifas-desarrolladores-latam-2026',
    'sourceType','rate_benchmark','regionsJson','["AR"]',
    'supportedServicesJson','["programming"]','priority',12,'enabled',1,
    'usageMode','market_price','acquisitionMode','auto_http','cooldownHours',24,
    'purpose','Bandas horarias para desarrolladores freelance de Argentina por experiencia.',
    'dataContribution','Rangos junior, intermedio y senior en USD/hora con país y fecha.',
    'appBenefit','Calcula el precio de mercado argentino sin mezclarlo con el mercado global.',
    'participatesInSuggestions',1,'automationStatus','APPROVED','adapterKey','prolatam',
    'currentStatus','READY','lastError',NULL,'businessSourceType','market',
    'marketCountry','Argentina','sourceCurrency','USD','sourceUpdatedAt','2026-05-19'
  ),
  'Bandas horarias para desarrolladores freelance de Argentina por experiencia.',
  'Rangos junior, intermedio y senior en USD/hora con país y fecha.',
  'Calcula el precio de mercado argentino sin mezclarlo con el mercado global.',
  1,'APPROVED','READY','prolatam','market','Argentina','USD','2026-05-19','automatic',
  strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
);

INSERT OR REPLACE INTO pricing_engine_sources (
  engine_id,source_id,role,preference,participates_in_suggestions,
  match_score_micros,explanation,assigned_by,created_at,updated_at
)
VALUES (
  'engine-video-editing','source-prolatam-video-ar','reference','preferred',1,980000,
  'Benchmark automático específico para edición de video en Argentina.',
  'automatic',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
);

INSERT OR REPLACE INTO pricing_engine_sources (
  engine_id,source_id,role,preference,participates_in_suggestions,
  match_score_micros,explanation,assigned_by,created_at,updated_at
)
VALUES (
  'engine-programming','source-prolatam-programming-ar','reference','preferred',1,980000,
  'Benchmark automático específico para desarrollo freelance en Argentina.',
  'automatic',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
);
