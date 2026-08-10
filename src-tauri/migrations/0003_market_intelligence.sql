PRAGMA foreign_keys = ON;

ALTER TABLE market_sources ADD COLUMN purpose TEXT;
ALTER TABLE market_sources ADD COLUMN data_contribution TEXT;
ALTER TABLE market_sources ADD COLUMN app_benefit TEXT;
ALTER TABLE market_sources ADD COLUMN participates_in_suggestions INTEGER NOT NULL DEFAULT 0 CHECK(participates_in_suggestions IN (0, 1));
ALTER TABLE market_sources ADD COLUMN automation_status TEXT NOT NULL DEFAULT 'MANUAL_ONLY' CHECK(automation_status IN ('APPROVED','UNREVIEWED','MANUAL_ONLY','BLOCKED'));
ALTER TABLE market_sources ADD COLUMN current_status TEXT NOT NULL DEFAULT 'MANUAL' CHECK(current_status IN ('READY','FETCHING','SUCCESS','CACHED','MANUAL','BLOCKED','ERROR','DISABLED','NEEDS_CONFIGURATION'));
ALTER TABLE market_sources ADD COLUMN adapter_key TEXT;
ALTER TABLE market_sources ADD COLUMN last_request_at TEXT;
ALTER TABLE market_sources ADD COLUMN last_success_at TEXT;
ALTER TABLE market_sources ADD COLUMN last_failure_at TEXT;
ALTER TABLE market_sources ADD COLUMN cooldown_until TEXT;
ALTER TABLE market_sources ADD COLUMN consecutive_failures INTEGER NOT NULL DEFAULT 0 CHECK(consecutive_failures >= 0);
ALTER TABLE market_sources ADD COLUMN last_http_status INTEGER;
ALTER TABLE market_sources ADD COLUMN last_error TEXT;
ALTER TABLE market_sources ADD COLUMN observation_count INTEGER NOT NULL DEFAULT 0 CHECK(observation_count >= 0);
ALTER TABLE market_sources ADD COLUMN archived_at TEXT;

UPDATE market_sources
SET purpose = CASE usage_mode
    WHEN 'market_price' THEN 'Referencia pública de precios o tarifas para servicios comparables.'
    WHEN 'salary_context' THEN 'Contexto salarial y de demanda laboral; no equivale a una tarifa freelance.'
    WHEN 'rate_methodology' THEN 'Metodología para construir y revisar una tarifa profesional sostenible.'
    WHEN 'currency' THEN 'Referencia oficial para conversiones monetarias auditables.'
    ELSE 'Contexto complementario para interpretar el mercado.' END,
    data_contribution = CASE usage_mode
    WHEN 'market_price' THEN 'Importes, rangos, moneda, unidad, región y fecha publicados por la fuente.'
    WHEN 'salary_context' THEN 'Rangos salariales, rol, experiencia, región, moneda y período.'
    WHEN 'rate_methodology' THEN 'Criterios, fórmulas y variables para calcular tarifas.'
    WHEN 'currency' THEN 'Par de monedas, cotización, fecha y organismo de origen.'
    ELSE 'Evidencia pública y metadata trazable.' END,
    app_benefit = CASE usage_mode
    WHEN 'market_price' THEN 'Permite contrastar el cálculo interno sin reemplazar el precio final.'
    WHEN 'salary_context' THEN 'Aporta contexto económico separado; nunca entra directo en la mediana freelance.'
    WHEN 'rate_methodology' THEN 'Ayuda a auditar los supuestos de economía y margen.'
    WHEN 'currency' THEN 'Convierte observaciones conservando valor, moneda, tasa y fecha originales.'
    ELSE 'Mejora la explicación de la sugerencia sin inventar precios.' END,
    acquisition_mode = CASE WHEN acquisition_mode = 'disabled' THEN 'manual' ELSE acquisition_mode END,
    current_status = CASE WHEN enabled = 1 THEN 'READY' ELSE 'DISABLED' END,
    automation_status = 'MANUAL_ONLY',
    cooldown_hours = COALESCE(cooldown_hours, 24);

UPDATE market_sources SET
  base_url='https://tarifario.org/multimedia-c27',
  source_type='professional_tariff', usage_mode='market_price',
  purpose='Aranceles orientativos para servicios creativos en Argentina.',
  data_contribution='Cliente A, B y C en ARS, por minuto o por proyecto, con descripción del servicio.',
  app_benefit='Compara la cotización audiovisual con referencias argentinas preservando el tipo de cliente definido por la fuente.',
  adapter_key='tarifario', acquisition_mode='manual', automation_status='BLOCKED', current_status='BLOCKED',
  participates_in_suggestions=1, enabled=1,
  last_error='El sitio público redirige actualmente a una página de cuenta suspendida.'
WHERE system_key='tarifario';

UPDATE market_sources SET
  base_url='https://www.yunojuno.com/blogs/day-rates-update-film-motion',
  source_type='rate_benchmark', usage_mode='market_price',
  regions_json='["UK","INTERNATIONAL"]', supported_services_json='["video-editing"]',
  purpose='Benchmark publicado de tarifas diarias para profesionales freelance de Film & Motion.',
  data_contribution='Rol, tarifa diaria original, moneda, año y disciplina.',
  app_benefit='Aporta contexto internacional por día; no se convierte automáticamente en precio de proyecto.',
  adapter_key='yunojuno', acquisition_mode='auto_http', automation_status='APPROVED', current_status='READY',
  participates_in_suggestions=0, enabled=1, priority=15, cooldown_hours=72
WHERE system_key='yunojuno';

UPDATE market_sources SET
  base_url='https://remotejobs.lat/tools/calculadora-salario-remoto-latam',
  source_type='job_board', usage_mode='salary_context',
  regions_json='["LATAM","INTERNATIONAL"]', supported_services_json='["programming"]',
  purpose='Rangos salariales mensuales para roles tecnológicos remotos en LATAM.',
  data_contribution='Rol, nivel, rango mensual en USD, región y fecha de consulta.',
  app_benefit='Muestra contexto de empleo remoto separado de las tarifas freelance.',
  adapter_key='remotejobs', acquisition_mode='auto_http', automation_status='APPROVED', current_status='READY',
  participates_in_suggestions=0, enabled=1, cooldown_hours=24
WHERE system_key='remotejobs-lat';

UPDATE market_sources SET
  base_url='https://api.bcra.gob.ar/estadisticascambiarias/v1.0/Cotizaciones',
  source_type='currency', usage_mode='currency',
  purpose='Cotización oficial de divisas publicada por el Banco Central de la República Argentina.',
  data_contribution='Par USD/ARS, tipo de cotización, fecha oficial y URL del endpoint.',
  app_benefit='Permite conversiones auditables sin pedir claves ni inventar un tipo de cambio.',
  adapter_key='bcra', acquisition_mode='auto_http', automation_status='APPROVED', current_status='READY',
  participates_in_suggestions=0, enabled=1, priority=5, cooldown_hours=24
WHERE system_key='bcra';

-- Homepages públicas para el flujo manual "Abrir fuente". Permanecen desactivadas
-- y MANUAL_ONLY: una URL conocida no implica una autorización de automatización.
UPDATE market_sources SET base_url = CASE system_key
  WHEN 'workana' THEN 'https://www.workana.com/'
  WHEN 'glassdoor' THEN 'https://www.glassdoor.com/'
  WHEN 'randstad' THEN 'https://www.randstad.com.ar/'
  WHEN 'michael-page' THEN 'https://www.michaelpage.com.ar/'
  WHEN 'upwork' THEN 'https://www.upwork.com/'
  WHEN 'twine' THEN 'https://www.twine.net/'
  WHEN 'malt' THEN 'https://www.malt.com/'
  WHEN 'contra' THEN 'https://contra.com/'
  WHEN 'freelancer' THEN 'https://www.freelancer.com/'
  WHEN 'peopleperhour' THEN 'https://www.peopleperhour.com/'
  WHEN 'fiverr' THEN 'https://www.fiverr.com/'
  WHEN 'toptal' THEN 'https://www.toptal.com/'
  WHEN 'guru' THEN 'https://www.guru.com/'
  WHEN 'codeable' THEN 'https://www.codeable.io/'
  WHEN 'levels' THEN 'https://www.levels.fyi/'
  WHEN 'payscale' THEN 'https://www.payscale.com/'
  WHEN 'indeed' THEN 'https://www.indeed.com/career/salaries'
  WHEN 'salary-com' THEN 'https://www.salary.com/'
  WHEN 'talent' THEN 'https://www.talent.com/salary'
  WHEN 'stackoverflow' THEN 'https://survey.stackoverflow.co/'
  WHEN 'ziprecruiter' THEN 'https://www.ziprecruiter.com/Salaries'
  WHEN 'comparably' THEN 'https://www.comparably.com/salaries'
  WHEN 'salaryexpert' THEN 'https://www.salaryexpert.com/'
  WHEN 'paylab' THEN 'https://www.paylab.com/'
  WHEN 'world-salaries' THEN 'https://worldsalaries.com/'
  WHEN 'salary-explorer' THEN 'https://www.salaryexplorer.com/'
  WHEN 'coroflot' THEN 'https://www.coroflot.com/designsalaryguide'
  WHEN 'aiga' THEN 'https://www.aiga.org/'
  WHEN '99designs' THEN 'https://99designs.com/'
  WHEN 'creativepool' THEN 'https://creativepool.com/'
  WHEN 'clockify' THEN 'https://clockify.me/hourly-rate-calculator'
  WHEN 'harvest' THEN 'https://www.getharvest.com/'
  WHEN 'bonsai' THEN 'https://www.hellobonsai.com/'
  WHEN 'timesheet' THEN 'https://www.timesheet.io/'
  ELSE base_url END
WHERE system_key IN (
  'workana','glassdoor','randstad','michael-page','upwork','twine','malt','contra',
  'freelancer','peopleperhour','fiverr','toptal','guru','codeable','levels','payscale',
  'indeed','salary-com','talent','stackoverflow','ziprecruiter','comparably','salaryexpert',
  'paylab','world-salaries','salary-explorer','coroflot','aiga','99designs','creativepool',
  'clockify','harvest','bonsai','timesheet'
);

CREATE TABLE IF NOT EXISTS market_observations (
  id TEXT PRIMARY KEY NOT NULL,
  source_id TEXT NOT NULL REFERENCES market_sources(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  origin TEXT NOT NULL CHECK(origin IN ('AUTO','MANUAL')),
  service_type TEXT NOT NULL,
  subservice TEXT,
  category TEXT,
  region TEXT NOT NULL,
  country TEXT,
  currency TEXT NOT NULL,
  price_type TEXT NOT NULL CHECK(price_type IN ('HOURLY','DAILY','PROJECT','PER_MINUTE','PER_ITEM','MONTHLY_SALARY','ANNUAL_SALARY','FIXED','RANGE','UNKNOWN')),
  unit TEXT NOT NULL,
  price_min_minor INTEGER,
  price_max_minor INTEGER,
  price_value_minor INTEGER,
  original_value_text TEXT NOT NULL,
  converted_value_minor INTEGER,
  converted_currency TEXT,
  exchange_rate_micros INTEGER,
  exchange_rate_date TEXT,
  exchange_rate_source TEXT,
  experience_level TEXT,
  client_tier TEXT,
  source_type TEXT NOT NULL,
  source_url TEXT NOT NULL,
  published_at TEXT,
  retrieved_at TEXT NOT NULL,
  parser_version TEXT NOT NULL,
  confidence TEXT NOT NULL CHECK(confidence IN ('HIGH','MEDIUM','LOW','REVIEW_REQUIRED')),
  comparison_eligibility TEXT NOT NULL CHECK(comparison_eligibility IN ('ELIGIBLE','CONTEXT_ONLY','REVIEW_REQUIRED','REJECTED','POSSIBLE_OUTLIER')),
  exclusion_reason TEXT,
  raw_fingerprint TEXT NOT NULL UNIQUE,
  evidence_snippet TEXT,
  notes TEXT,
  created_at TEXT NOT NULL,
  CHECK(price_min_minor IS NULL OR price_min_minor >= 0),
  CHECK(price_max_minor IS NULL OR price_max_minor >= 0),
  CHECK(price_value_minor IS NULL OR price_value_minor >= 0),
  CHECK(price_min_minor IS NULL OR price_max_minor IS NULL OR price_min_minor <= price_max_minor)
);

CREATE INDEX IF NOT EXISTS idx_market_observations_source ON market_observations(source_id, retrieved_at DESC);
CREATE INDEX IF NOT EXISTS idx_market_observations_compare ON market_observations(service_type, region, currency, price_type, retrieved_at DESC);

CREATE TABLE IF NOT EXISTS market_snapshots (
  id TEXT PRIMARY KEY NOT NULL,
  quote_id TEXT REFERENCES quotes(id) ON UPDATE CASCADE ON DELETE SET NULL,
  quote_service_id TEXT REFERENCES quote_services(id) ON UPDATE CASCADE ON DELETE SET NULL,
  query_context_json TEXT NOT NULL,
  currency TEXT NOT NULL CHECK(currency IN ('ARS','USD')),
  observation_count INTEGER NOT NULL,
  comparable_observation_count INTEGER NOT NULL,
  source_count INTEGER NOT NULL,
  minimum_filtered_minor INTEGER,
  p25_minor INTEGER,
  market_median_minor INTEGER,
  p75_minor INTEGER,
  maximum_filtered_minor INTEGER,
  confidence_level TEXT NOT NULL CHECK(confidence_level IN ('HIGH','MEDIUM','LOW','INSUFFICIENT')),
  calculated_price_minor INTEGER,
  suggested_price_minor INTEGER,
  final_price_minor_at_creation INTEGER,
  summary_json TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_market_snapshots_service ON market_snapshots(quote_service_id, created_at DESC);

CREATE TABLE IF NOT EXISTS market_snapshot_observations (
  snapshot_id TEXT NOT NULL REFERENCES market_snapshots(id) ON UPDATE CASCADE ON DELETE CASCADE,
  observation_id TEXT NOT NULL REFERENCES market_observations(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  included INTEGER NOT NULL CHECK(included IN (0,1)),
  exclusion_reason TEXT,
  normalized_value_minor INTEGER,
  converted_value_minor INTEGER,
  converted_currency TEXT,
  exchange_rate_micros INTEGER,
  exchange_rate_date TEXT,
  exchange_rate_source TEXT,
  PRIMARY KEY(snapshot_id, observation_id)
);

CREATE TABLE IF NOT EXISTS market_fetch_logs (
  id TEXT PRIMARY KEY NOT NULL,
  source_id TEXT NOT NULL REFERENCES market_sources(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  url TEXT NOT NULL,
  method TEXT NOT NULL,
  started_at TEXT NOT NULL,
  finished_at TEXT NOT NULL,
  status TEXT NOT NULL,
  http_status INTEGER,
  duration_ms INTEGER NOT NULL,
  cache_hit INTEGER NOT NULL CHECK(cache_hit IN (0,1)),
  observation_count INTEGER NOT NULL,
  error_type TEXT,
  error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_market_fetch_logs_source ON market_fetch_logs(source_id, started_at DESC);

CREATE TABLE IF NOT EXISTS market_fx_rates (
  id TEXT PRIMARY KEY NOT NULL,
  source_id TEXT NOT NULL REFERENCES market_sources(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  base_currency TEXT NOT NULL,
  quote_currency TEXT NOT NULL,
  rate_micros INTEGER NOT NULL CHECK(rate_micros > 0),
  rate_date TEXT NOT NULL,
  source_url TEXT NOT NULL,
  retrieved_at TEXT NOT NULL,
  UNIQUE(source_id, base_currency, quote_currency, rate_date)
);

UPDATE market_sources SET default_data_json = json_object(
  'name', name, 'baseUrl', base_url, 'sourceType', source_type,
  'regionsJson', regions_json, 'supportedServicesJson', supported_services_json,
  'priority', priority, 'enabled', enabled, 'usageMode', usage_mode,
  'acquisitionMode', acquisition_mode, 'cooldownHours', cooldown_hours,
  'purpose', purpose, 'dataContribution', data_contribution, 'appBenefit', app_benefit,
  'participatesInSuggestions', participates_in_suggestions,
  'automationStatus', automation_status, 'adapterKey', adapter_key
  , 'currentStatus', current_status, 'lastError', last_error
) WHERE is_system_source=1;
