PRAGMA foreign_keys = ON;

-- Una investigación de mercado es evidencia, no una edición del alcance del
-- usuario. Estos campos documentan si su sugerencia se pudo aplicar al
-- borrador que originó la consulta.
ALTER TABLE market_snapshots ADD COLUMN base_service_revision INTEGER;
ALTER TABLE market_snapshots ADD COLUMN suggestion_update_status TEXT NOT NULL DEFAULT 'LEGACY'
  CHECK(suggestion_update_status IN ('LEGACY','PENDING','APPLIED','SKIPPED_DRAFT_CHANGED','INSUFFICIENT','DISABLED'));
ALTER TABLE market_snapshots ADD COLUMN suggestion_update_message TEXT;

-- Corrige datos de catálogo o configuraciones antiguas contradictorias. Los
-- salarios, moneda, metodología y contexto pueden mostrarse, pero nunca
-- participan en una sugerencia de precio.
UPDATE market_sources
SET participates_in_suggestions = 0
WHERE usage_mode <> 'market_price'
   OR source_type IN ('salary','job_board','methodology','currency');

UPDATE pricing_engine_sources
SET participates_in_suggestions = 0
WHERE role <> 'reference'
   OR source_id IN (
     SELECT id FROM market_sources
     WHERE usage_mode <> 'market_price'
        OR source_type IN ('salary','job_board','methodology','currency')
   );

-- Catálogo honesto: estas fuentes no se consultan automáticamente ni se
-- presentan como precios comparables. Tarifario.org conservaba una URL rota;
-- por eso se quita también del default usado por "Restaurar catálogo".
UPDATE market_sources
SET name='Tarifario.org · registro manual',
    base_url=NULL,
    source_type='professional_tariff',
    regions_json='["AR","LATAM"]',
    supported_services_json='["video-editing","programming"]',
    priority=10,
    enabled=1,
    usage_mode='context_only',
    acquisition_mode='manual',
    cooldown_hours=0,
    purpose='Registro manual de aranceles que vos verifiques; el sitio público anterior ya no se consulta.',
    data_contribution='Observaciones manuales con importe, unidad, moneda, fecha y evidencia que cargues vos.',
    app_benefit='Conserva antecedentes como contexto y nunca genera una sugerencia automática.',
    participates_in_suggestions=0,
    automation_status='MANUAL_ONLY',
    current_status='MANUAL',
    adapter_key=NULL,
    last_error='No hay automatización activa: la URL pública anterior no es una fuente verificable.',
    default_data_json=json_object(
      'name','Tarifario.org · registro manual', 'baseUrl',NULL,
      'sourceType','professional_tariff', 'regionsJson','["AR","LATAM"]',
      'supportedServicesJson','["video-editing","programming"]', 'priority',10,
      'enabled',1, 'usageMode','context_only', 'acquisitionMode','manual', 'cooldownHours',0,
      'purpose','Registro manual de aranceles que vos verifiques; el sitio público anterior ya no se consulta.',
      'dataContribution','Observaciones manuales con importe, unidad, moneda, fecha y evidencia que cargues vos.',
      'appBenefit','Conserva antecedentes como contexto y nunca genera una sugerencia automática.',
      'participatesInSuggestions',0, 'automationStatus','MANUAL_ONLY', 'adapterKey',NULL,
      'currentStatus','MANUAL', 'lastError','No hay automatización activa: la URL pública anterior no es una fuente verificable.',
      'businessSourceType','market', 'marketCountry','Argentina', 'sourceCurrency','ARS'
    )
WHERE system_key='tarifario';

UPDATE market_sources
SET name='YunoJuno · contexto',
    base_url='https://www.yunojuno.com/blogs/day-rates-update-film-motion',
    source_type='rate_benchmark',
    regions_json='["UK","INTERNATIONAL"]',
    supported_services_json='["video-editing"]',
    priority=15,
    enabled=1,
    usage_mode='context_only',
    acquisition_mode='manual',
    cooldown_hours=0,
    purpose='Contexto público de tarifas diarias de Film & Motion; no se extrae automáticamente.',
    data_contribution='Referencias diarias en GBP conservadas con su unidad original.',
    app_benefit='Sirve para contexto internacional, nunca para simular un precio por proyecto.',
    participates_in_suggestions=0,
    automation_status='MANUAL_ONLY',
    current_status='MANUAL',
    adapter_key=NULL,
    last_error=NULL,
    default_data_json=json_object(
      'name','YunoJuno · contexto', 'baseUrl','https://www.yunojuno.com/blogs/day-rates-update-film-motion',
      'sourceType','rate_benchmark', 'regionsJson','["UK","INTERNATIONAL"]',
      'supportedServicesJson','["video-editing"]', 'priority',15,
      'enabled',1, 'usageMode','context_only', 'acquisitionMode','manual', 'cooldownHours',0,
      'purpose','Contexto público de tarifas diarias de Film & Motion; no se extrae automáticamente.',
      'dataContribution','Referencias diarias en GBP conservadas con su unidad original.',
      'appBenefit','Sirve para contexto internacional, nunca para simular un precio por proyecto.',
      'participatesInSuggestions',0, 'automationStatus','MANUAL_ONLY', 'adapterKey',NULL,
      'currentStatus','MANUAL', 'lastError',NULL,
      'businessSourceType','market', 'marketCountry','United Kingdom', 'sourceCurrency','GBP'
    )
WHERE system_key='yunojuno';

UPDATE market_sources
SET name='RemoteJobs.lat · contexto salarial',
    base_url='https://remotejobs.lat/tools/calculadora-salario-remoto-latam',
    source_type='job_board',
    regions_json='["LATAM","INTERNATIONAL"]',
    supported_services_json='["programming"]',
    priority=70,
    enabled=1,
    usage_mode='salary_context',
    acquisition_mode='manual',
    cooldown_hours=0,
    purpose='Contexto salarial de empleo remoto; no equivale a una tarifa freelance.',
    data_contribution='Rangos salariales que se conservan separados de las referencias por proyecto.',
    app_benefit='Ayuda a interpretar el mercado laboral, sin modificar ni sugerir el precio de una cotización.',
    participates_in_suggestions=0,
    automation_status='MANUAL_ONLY',
    current_status='MANUAL',
    adapter_key=NULL,
    last_error=NULL,
    default_data_json=json_object(
      'name','RemoteJobs.lat · contexto salarial', 'baseUrl','https://remotejobs.lat/tools/calculadora-salario-remoto-latam',
      'sourceType','job_board', 'regionsJson','["LATAM","INTERNATIONAL"]',
      'supportedServicesJson','["programming"]', 'priority',70,
      'enabled',1, 'usageMode','salary_context', 'acquisitionMode','manual', 'cooldownHours',0,
      'purpose','Contexto salarial de empleo remoto; no equivale a una tarifa freelance.',
      'dataContribution','Rangos salariales que se conservan separados de las referencias por proyecto.',
      'appBenefit','Ayuda a interpretar el mercado laboral, sin modificar ni sugerir el precio de una cotización.',
      'participatesInSuggestions',0, 'automationStatus','MANUAL_ONLY', 'adapterKey',NULL,
      'currentStatus','MANUAL', 'lastError',NULL,
      'businessSourceType','market', 'marketCountry','LATAM', 'sourceCurrency','USD'
    )
WHERE system_key='remotejobs-lat';

-- BCRA permanece como la única consulta automática de catálogo: aporta sólo la
-- tasa USD/ARS atribuida a la API oficial, nunca un precio de servicio.
UPDATE market_sources
SET name='BCRA · tipo de cambio oficial',
    base_url='https://api.bcra.gob.ar/estadisticascambiarias/v1.0/Cotizaciones',
    source_type='currency',
    regions_json='["AR"]',
    supported_services_json='["video-editing","programming"]',
    priority=5,
    enabled=1,
    usage_mode='currency',
    acquisition_mode='auto_http',
    cooldown_hours=24,
    purpose='Tipo de cambio USD/ARS publicado por el Banco Central de la República Argentina.',
    data_contribution='Par USD/ARS, fecha oficial y URL de la API del BCRA.',
    app_benefit='Convierte observaciones manteniendo la tasa y su atribución; no sugiere precios de servicios.',
    participates_in_suggestions=0,
    automation_status='APPROVED',
    current_status='READY',
    adapter_key='bcra',
    last_error=NULL,
    default_data_json=json_object(
      'name','BCRA · tipo de cambio oficial', 'baseUrl','https://api.bcra.gob.ar/estadisticascambiarias/v1.0/Cotizaciones',
      'sourceType','currency', 'regionsJson','["AR"]',
      'supportedServicesJson','["video-editing","programming"]', 'priority',5,
      'enabled',1, 'usageMode','currency', 'acquisitionMode','auto_http', 'cooldownHours',24,
      'purpose','Tipo de cambio USD/ARS publicado por el Banco Central de la República Argentina.',
      'dataContribution','Par USD/ARS, fecha oficial y URL de la API del BCRA.',
      'appBenefit','Convierte observaciones manteniendo la tasa y su atribución; no sugiere precios de servicios.',
      'participatesInSuggestions',0, 'automationStatus','APPROVED', 'adapterKey','bcra',
      'currentStatus','READY', 'lastError',NULL,
      'businessSourceType','market', 'marketCountry','Argentina', 'sourceCurrency','ARS'
    )
WHERE system_key='bcra';

UPDATE pricing_engine_sources
SET participates_in_suggestions = 0
WHERE source_id IN ('source-tarifario','source-yunojuno','source-remotejobs-lat','source-bcra');
