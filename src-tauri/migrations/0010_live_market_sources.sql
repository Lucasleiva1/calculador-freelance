PRAGMA foreign_keys = ON;

-- Fuentes automáticas verificadas en agosto de 2026. Sólo se automatizan
-- páginas públicas que exponen unidad, moneda y metodología suficientes para
-- conservar evidencia. Los sitios rotos, con login o sin datos auditables
-- permanecen manuales o desactivados.

UPDATE market_sources
SET name='Upwork · rangos freelance',
    base_url='https://www.upwork.com/hire/video-editors/cost/',
    source_type='freelance_marketplace',
    regions_json='["GLOBAL"]',
    supported_services_json='["video-editing","programming"]',
    priority=10,
    enabled=1,
    usage_mode='context_only',
    acquisition_mode='manual',
    cooldown_hours=0,
    purpose='Rangos públicos de contratación freelance por hora para edición de video y desarrollo de software.',
    data_contribution='Rango general y rangos por experiencia en USD/hora, con página y fecha del benchmark.',
    app_benefit='Contrasta las horas del proyecto con referencias públicas sin reemplazar tu tarifa interna ni tu precio final.',
    participates_in_suggestions=0,
    automation_status='BLOCKED',
    current_status='BLOCKED',
    adapter_key=NULL,
    last_error='Upwork devuelve un desafio anti-bot a la aplicacion; se conserva como referencia manual y nunca se simula una actualizacion.',
    consecutive_failures=0,
    business_source_type='market',
    market_country='Global',
    source_currency='USD',
    source_updated_at='2026-01-01',
    default_data_json=json_object(
      'name','Upwork · rangos freelance', 'baseUrl','https://www.upwork.com/hire/video-editors/cost/',
      'sourceType','freelance_marketplace', 'regionsJson','["GLOBAL"]',
      'supportedServicesJson','["video-editing","programming"]', 'priority',10,
      'enabled',1, 'usageMode','context_only', 'acquisitionMode','manual', 'cooldownHours',0,
      'purpose','Rangos públicos de contratación freelance por hora para edición de video y desarrollo de software.',
      'dataContribution','Rango general y rangos por experiencia en USD/hora, con página y fecha del benchmark.',
      'appBenefit','Contrasta las horas del proyecto con referencias públicas sin reemplazar tu tarifa interna ni tu precio final.',
      'participatesInSuggestions',0, 'automationStatus','BLOCKED', 'adapterKey',NULL,
      'currentStatus','BLOCKED', 'lastError','Upwork devuelve un desafio anti-bot a la aplicacion; se conserva como referencia manual.',
      'businessSourceType','market', 'marketCountry','Global', 'sourceCurrency','USD',
      'sourceUpdatedAt','2026-01-01'
    ),
    updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE system_key='upwork';

UPDATE market_sources
SET name='YunoJuno · informe 2026',
    base_url='https://www.yunojuno.com/freelancer-rates-report',
    source_type='rate_benchmark',
    regions_json='["GLOBAL","INTERNATIONAL"]',
    supported_services_json='["video-editing","programming"]',
    priority=15,
    enabled=1,
    usage_mode='market_price',
    acquisition_mode='auto_http',
    cooldown_hours=72,
    purpose='Benchmark global 2026 de contratistas y freelancers para disciplinas creativas y software.',
    data_contribution='Promedios en USD/hora y GBP/día basados en más de 182.000 datos de 2024 y 2025.',
    app_benefit='Aporta una segunda referencia profesional y separa la tarifa horaria comparable de la tarifa diaria contextual.',
    participates_in_suggestions=1,
    automation_status='APPROVED',
    current_status='READY',
    adapter_key='yunojuno',
    last_error=NULL,
    consecutive_failures=0,
    business_source_type='market',
    market_country='Global',
    source_currency='USD',
    source_updated_at='2026-01-01',
    default_data_json=json_object(
      'name','YunoJuno · informe 2026', 'baseUrl','https://www.yunojuno.com/freelancer-rates-report',
      'sourceType','rate_benchmark', 'regionsJson','["GLOBAL","INTERNATIONAL"]',
      'supportedServicesJson','["video-editing","programming"]', 'priority',15,
      'enabled',1, 'usageMode','market_price', 'acquisitionMode','auto_http', 'cooldownHours',72,
      'purpose','Benchmark global 2026 de contratistas y freelancers para disciplinas creativas y software.',
      'dataContribution','Promedios en USD/hora y GBP/día basados en más de 182.000 datos de 2024 y 2025.',
      'appBenefit','Aporta una segunda referencia profesional y separa la tarifa horaria comparable de la tarifa diaria contextual.',
      'participatesInSuggestions',1, 'automationStatus','APPROVED', 'adapterKey','yunojuno',
      'currentStatus','READY', 'lastError',NULL,
      'businessSourceType','market', 'marketCountry','Global', 'sourceCurrency','USD',
      'sourceUpdatedAt','2026-01-01'
    ),
    updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE system_key='yunojuno';

INSERT OR IGNORE INTO market_sources (
  id,name,base_url,source_type,regions_json,supported_services_json,priority,enabled,
  usage_mode,acquisition_mode,cooldown_hours,notes,is_system_source,system_key,
  default_data_json,purpose,data_contribution,app_benefit,participates_in_suggestions,
  automation_status,current_status,adapter_key,business_source_type,market_country,
  source_currency,source_updated_at,classification_origin,created_at,updated_at
) VALUES (
  'source-reelrate','ReelRate - video 2026','https://reel-rate.com/','rate_benchmark',
  '["GLOBAL"]','["video-editing"]',18,1,'market_price','auto_http',24,
  'Benchmark especifico de edicion de video, actualizado en agosto de 2026.',1,'reelrate',
  json_object(
    'name','ReelRate - video 2026','baseUrl','https://reel-rate.com/','sourceType','rate_benchmark',
    'regionsJson','["GLOBAL"]','supportedServicesJson','["video-editing"]','priority',18,
    'enabled',1,'usageMode','market_price','acquisitionMode','auto_http','cooldownHours',24,
    'purpose','Rangos horarios 2026 para editores de video por experiencia.',
    'dataContribution','Rangos junior, intermedio y senior en USD/hora, fecha y enlace.',
    'appBenefit','Contrasta el calculo de video con un benchmark especifico y actual.',
    'participatesInSuggestions',1,'automationStatus','APPROVED','adapterKey','reelrate',
    'currentStatus','READY','lastError',NULL,'businessSourceType','market',
    'marketCountry','Global','sourceCurrency','USD','sourceUpdatedAt','2026-08-01'
  ),
  'Rangos horarios 2026 para editores de video por experiencia.',
  'Rangos junior, intermedio y senior en USD/hora, fecha y enlace.',
  'Contrasta el calculo de video con un benchmark especifico y actual.',
  1,'APPROVED','READY','reelrate','market','Global','USD','2026-08-01','automatic',
  strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
);

INSERT OR IGNORE INTO market_sources (
  id,name,base_url,source_type,regions_json,supported_services_json,priority,enabled,
  usage_mode,acquisition_mode,cooldown_hours,notes,is_system_source,system_key,
  default_data_json,purpose,data_contribution,app_benefit,participates_in_suggestions,
  automation_status,current_status,adapter_key,business_source_type,market_country,
  source_currency,source_updated_at,classification_origin,created_at,updated_at
) VALUES (
  'source-indexdev','Index.dev - desarrollo 2026','https://www.index.dev/blog/freelance-developer-rates','rate_benchmark',
  '["GLOBAL","INTERNATIONAL"]','["programming"]',18,1,'market_price','auto_http',24,
  'Benchmark internacional de tarifas freelance de desarrollo, publicado en 2026.',1,'indexdev',
  json_object(
    'name','Index.dev - desarrollo 2026','baseUrl','https://www.index.dev/blog/freelance-developer-rates','sourceType','rate_benchmark',
    'regionsJson','["GLOBAL","INTERNATIONAL"]','supportedServicesJson','["programming"]','priority',18,
    'enabled',1,'usageMode','market_price','acquisitionMode','auto_http','cooldownHours',24,
    'purpose','Rangos horarios 2026 para desarrolladores de software por experiencia.',
    'dataContribution','Rangos entry, mid y senior en USD/hora, fecha y enlace.',
    'appBenefit','Contrasta el calculo de programacion con un benchmark internacional actual.',
    'participatesInSuggestions',1,'automationStatus','APPROVED','adapterKey','indexdev',
    'currentStatus','READY','lastError',NULL,'businessSourceType','market',
    'marketCountry','Global','sourceCurrency','USD','sourceUpdatedAt','2026-06-01'
  ),
  'Rangos horarios 2026 para desarrolladores de software por experiencia.',
  'Rangos entry, mid y senior en USD/hora, fecha y enlace.',
  'Contrasta el calculo de programacion con un benchmark internacional actual.',
  1,'APPROVED','READY','indexdev','market','Global','USD','2026-06-01','automatic',
  strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
);

INSERT OR IGNORE INTO market_sources (
  id,name,base_url,source_type,regions_json,supported_services_json,priority,enabled,
  usage_mode,acquisition_mode,cooldown_hours,notes,is_system_source,system_key,
  default_data_json,purpose,data_contribution,app_benefit,participates_in_suggestions,
  automation_status,current_status,adapter_key,business_source_type,market_country,
  source_currency,source_updated_at,classification_origin,created_at,updated_at
) VALUES (
  'source-remoteok','Remote OK · salarios publicados','https://remoteok.com/api','job_board',
  '["GLOBAL"]','["video-editing","programming"]',80,1,'salary_context','auto_http',6,
  'La API exige atribución y enlace; cada observación conserva el aviso original.',1,'remoteok',
  json_object(
    'name','Remote OK · salarios publicados', 'baseUrl','https://remoteok.com/api',
    'sourceType','job_board', 'regionsJson','["GLOBAL"]',
    'supportedServicesJson','["video-editing","programming"]', 'priority',80,
    'enabled',1, 'usageMode','salary_context', 'acquisitionMode','auto_http', 'cooldownHours',6,
    'purpose','Avisos remotos recientes con rangos salariales publicados.',
    'dataContribution','Rol, rango salarial anual en USD, ubicación, fecha y enlace al aviso.',
    'appBenefit','Muestra demanda y salarios recientes como contexto separado de la tarifa freelance.',
    'participatesInSuggestions',0, 'automationStatus','APPROVED', 'adapterKey','remoteok',
    'currentStatus','READY', 'lastError',NULL,
    'businessSourceType','market', 'marketCountry','Global', 'sourceCurrency','USD'
  ),
  'Avisos remotos recientes con rangos salariales publicados.',
  'Rol, rango salarial anual en USD, ubicación, fecha y enlace al aviso.',
  'Muestra demanda y salarios recientes como contexto separado de la tarifa freelance.',
  0,'APPROVED','READY','remoteok','market','Global','USD',NULL,'automatic',
  strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
);

UPDATE pricing_engine_sources
SET role='reference', preference='preferred', participates_in_suggestions=1,
    match_score_micros=950000,
    explanation='Benchmark automático público con moneda, unidad y fecha verificables.',
    assigned_by='automatic', updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE source_id IN ('source-yunojuno')
  AND engine_id IN ('engine-video-editing','engine-programming');

UPDATE pricing_engine_sources
SET role='context', preference='available', participates_in_suggestions=0,
    explanation='La pagina bloquea consultas automaticas; queda disponible solo para evidencia manual.',
    assigned_by='automatic', updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE source_id='source-upwork'
  AND engine_id IN ('engine-video-editing','engine-programming');

INSERT OR REPLACE INTO pricing_engine_sources (
  engine_id,source_id,role,preference,participates_in_suggestions,
  match_score_micros,explanation,assigned_by,created_at,updated_at
)
SELECT id,'source-reelrate','reference','preferred',1,950000,
       'Benchmark automatico actual y especifico para edicion de video.',
       'automatic',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
FROM pricing_engines WHERE engine_key='video-editing';

INSERT OR REPLACE INTO pricing_engine_sources (
  engine_id,source_id,role,preference,participates_in_suggestions,
  match_score_micros,explanation,assigned_by,created_at,updated_at
)
SELECT id,'source-indexdev','reference','preferred',1,950000,
       'Benchmark automatico actual para desarrollo de software.',
       'automatic',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
FROM pricing_engines WHERE engine_key='programming';

INSERT OR REPLACE INTO pricing_engine_sources (
  engine_id,source_id,role,preference,participates_in_suggestions,
  match_score_micros,explanation,assigned_by,created_at,updated_at
)
SELECT id,'source-remoteok','context','available',0,800000,
       'Contexto salarial reciente; nunca participa en la sugerencia freelance.',
       'automatic',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
FROM pricing_engines
WHERE engine_key IN ('video-editing','programming');

-- El endpoint de RemoteJobs.lat no respondió de forma confiable durante la
-- verificación. Se conserva manual para no simular disponibilidad automática.
UPDATE market_sources
SET last_error='La página pública no respondió de forma confiable; se mantiene manual.',
    updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE system_key='remotejobs-lat' AND acquisition_mode='manual';
