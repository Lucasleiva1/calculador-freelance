PRAGMA foreign_keys = ON;

-- YunoJuno publica un informe util, pero Cloudflare no permite una consulta
-- estable desde la aplicacion. Se conserva visible para evidencia manual y
-- se reemplaza en sugerencias por fuentes que el cliente HTTP puede auditar.
UPDATE market_sources
SET usage_mode='context_only', acquisition_mode='manual', cooldown_hours=0,
    participates_in_suggestions=0, automation_status='BLOCKED',
    current_status='BLOCKED', adapter_key=NULL,
    last_error='Cloudflare impide una consulta automatica estable; la fuente queda disponible para evidencia manual.',
    default_data_json=json_set(
      default_data_json,
      '$.usageMode','context_only', '$.acquisitionMode','manual', '$.cooldownHours',0,
      '$.participatesInSuggestions',0, '$.automationStatus','BLOCKED',
      '$.adapterKey',NULL, '$.currentStatus','BLOCKED',
      '$.lastError','Cloudflare impide una consulta automatica estable; la fuente queda manual.'
    ),
    updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE system_key='yunojuno';

UPDATE pricing_engine_sources
SET role='context', preference='available', participates_in_suggestions=0,
    explanation='Fuente publica bloqueada por Cloudflare; solo evidencia manual.',
    assigned_by='automatic', updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE source_id='source-yunojuno';

-- Remote OK ofrece contexto salarial util para programacion, pero no suele
-- publicar avisos de video con salario; se evita mostrar un falso error alli.
UPDATE market_sources
SET supported_services_json='["programming"]',
    default_data_json=json_set(default_data_json,'$.supportedServicesJson','["programming"]'),
    updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE system_key='remoteok';

DELETE FROM pricing_engine_sources
WHERE source_id='source-remoteok' AND engine_id='engine-video-editing';

INSERT OR IGNORE INTO market_sources (
  id,name,base_url,source_type,regions_json,supported_services_json,priority,enabled,
  usage_mode,acquisition_mode,cooldown_hours,notes,is_system_source,system_key,
  default_data_json,purpose,data_contribution,app_benefit,participates_in_suggestions,
  automation_status,current_status,adapter_key,business_source_type,market_country,
  source_currency,source_updated_at,classification_origin,created_at,updated_at
) VALUES (
  'source-solopricing','SoloPricing - video 2026','https://www.solopricing.com/video-editor-rates-2026','rate_benchmark',
  '["GLOBAL"]','["video-editing"]',19,1,'market_price','auto_http',24,
  'Rangos de edicion de video publicados y actualizados en marzo de 2026.',1,'solopricing',
  json_object(
    'name','SoloPricing - video 2026','baseUrl','https://www.solopricing.com/video-editor-rates-2026',
    'sourceType','rate_benchmark','regionsJson','["GLOBAL"]',
    'supportedServicesJson','["video-editing"]','priority',19,'enabled',1,
    'usageMode','market_price','acquisitionMode','auto_http','cooldownHours',24,
    'purpose','Rangos horarios 2026 para editores de video por experiencia.',
    'dataContribution','Rangos entry, mid y senior en USD/hora, fecha y enlace.',
    'appBenefit','Segunda referencia actual para contrastar precios de video.',
    'participatesInSuggestions',1,'automationStatus','APPROVED','adapterKey','solopricing',
    'currentStatus','READY','lastError',NULL,'businessSourceType','market',
    'marketCountry','Global','sourceCurrency','USD','sourceUpdatedAt','2026-03-10'
  ),
  'Rangos horarios 2026 para editores de video por experiencia.',
  'Rangos entry, mid y senior en USD/hora, fecha y enlace.',
  'Segunda referencia actual para contrastar precios de video.',
  1,'APPROVED','READY','solopricing','market','Global','USD','2026-03-10','automatic',
  strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
);

INSERT OR IGNORE INTO market_sources (
  id,name,base_url,source_type,regions_json,supported_services_json,priority,enabled,
  usage_mode,acquisition_mode,cooldown_hours,notes,is_system_source,system_key,
  default_data_json,purpose,data_contribution,app_benefit,participates_in_suggestions,
  automation_status,current_status,adapter_key,business_source_type,market_country,
  source_currency,source_updated_at,classification_origin,created_at,updated_at
) VALUES (
  'source-golance','goLance - desarrollo 2026','https://golance.com/hiring/best-freelance-software-developers-hourly-rate','rate_benchmark',
  '["GLOBAL","INTERNATIONAL"]','["programming"]',19,1,'market_price','auto_http',24,
  'Rangos de desarrollo freelance publicados para 2026.',1,'golance',
  json_object(
    'name','goLance - desarrollo 2026','baseUrl','https://golance.com/hiring/best-freelance-software-developers-hourly-rate',
    'sourceType','rate_benchmark','regionsJson','["GLOBAL","INTERNATIONAL"]',
    'supportedServicesJson','["programming"]','priority',19,'enabled',1,
    'usageMode','market_price','acquisitionMode','auto_http','cooldownHours',24,
    'purpose','Rangos horarios 2026 para desarrolladores freelance por experiencia.',
    'dataContribution','Rangos junior, mid, senior y expert en USD/hora, fecha y enlace.',
    'appBenefit','Segunda referencia actual para contrastar precios de programacion.',
    'participatesInSuggestions',1,'automationStatus','APPROVED','adapterKey','golance',
    'currentStatus','READY','lastError',NULL,'businessSourceType','market',
    'marketCountry','Global','sourceCurrency','USD','sourceUpdatedAt','2026-01-01'
  ),
  'Rangos horarios 2026 para desarrolladores freelance por experiencia.',
  'Rangos junior, mid, senior y expert en USD/hora, fecha y enlace.',
  'Segunda referencia actual para contrastar precios de programacion.',
  1,'APPROVED','READY','golance','market','Global','USD','2026-01-01','automatic',
  strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
);

INSERT OR REPLACE INTO pricing_engine_sources (
  engine_id,source_id,role,preference,participates_in_suggestions,
  match_score_micros,explanation,assigned_by,created_at,updated_at
)
VALUES (
  'engine-video-editing','source-solopricing','reference','preferred',1,950000,
  'Benchmark automatico actual para edicion de video.','automatic',
  strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
);

INSERT OR REPLACE INTO pricing_engine_sources (
  engine_id,source_id,role,preference,participates_in_suggestions,
  match_score_micros,explanation,assigned_by,created_at,updated_at
)
VALUES (
  'engine-programming','source-golance','reference','preferred',1,950000,
  'Benchmark automatico actual para desarrollo de software.','automatic',
  strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
);
