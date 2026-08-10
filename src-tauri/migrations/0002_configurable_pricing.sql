PRAGMA foreign_keys = ON;

ALTER TABLE app_settings ADD COLUMN suggestions_enabled INTEGER NOT NULL DEFAULT 1 CHECK(suggestions_enabled IN (0, 1));
ALTER TABLE app_settings ADD COLUMN suggestion_strategy TEXT NOT NULL DEFAULT 'balanced' CHECK(suggestion_strategy IN ('competitive', 'balanced', 'premium'));
ALTER TABLE app_settings ADD COLUMN base_currency TEXT NOT NULL DEFAULT 'USD' CHECK(base_currency IN ('ARS', 'USD'));

ALTER TABLE quote_services ADD COLUMN suggested_subtotal_minor INTEGER CHECK(suggested_subtotal_minor IS NULL OR suggested_subtotal_minor >= 0);
ALTER TABLE quote_services ADD COLUMN final_subtotal_minor INTEGER CHECK(final_subtotal_minor IS NULL OR final_subtotal_minor >= 0);
ALTER TABLE quote_services ADD COLUMN has_override INTEGER NOT NULL DEFAULT 0 CHECK(has_override IN (0, 1));
ALTER TABLE quote_services ADD COLUMN pricing_snapshot_json TEXT;
ALTER TABLE quote_services ADD COLUMN service_definition_version INTEGER;
ALTER TABLE service_presets ADD COLUMN definition_version INTEGER NOT NULL DEFAULT 1;

UPDATE quote_services
SET final_subtotal_minor = COALESCE(manual_subtotal_minor, calculated_subtotal_minor),
    has_override = CASE WHEN manual_subtotal_minor IS NOT NULL THEN 1 ELSE 0 END;

CREATE TABLE IF NOT EXISTS service_definitions (
  id TEXT PRIMARY KEY NOT NULL,
  service_type TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  description TEXT,
  version INTEGER NOT NULL DEFAULT 1 CHECK(version > 0),
  enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
  suggestions_enabled INTEGER NOT NULL DEFAULT 1 CHECK(suggestions_enabled IN (0, 1)),
  default_strategy TEXT NOT NULL DEFAULT 'balanced' CHECK(default_strategy IN ('competitive', 'balanced', 'premium')),
  competitive_margin_micros INTEGER CHECK(competitive_margin_micros IS NULL OR (competitive_margin_micros >= 0 AND competitive_margin_micros < 1000000)),
  balanced_margin_micros INTEGER CHECK(balanced_margin_micros IS NULL OR (balanced_margin_micros >= 0 AND balanced_margin_micros < 1000000)),
  premium_margin_micros INTEGER CHECK(premium_margin_micros IS NULL OR (premium_margin_micros >= 0 AND premium_margin_micros < 1000000)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS service_parameters (
  id TEXT PRIMARY KEY NOT NULL,
  service_definition_id TEXT NOT NULL REFERENCES service_definitions(id) ON UPDATE CASCADE ON DELETE CASCADE,
  parameter_key TEXT NOT NULL,
  name TEXT NOT NULL,
  label TEXT NOT NULL,
  parameter_type TEXT NOT NULL CHECK(parameter_type IN ('single_select','multi_select','boolean','number','duration','currency','percentage','text')),
  description TEXT,
  required INTEGER NOT NULL DEFAULT 0 CHECK(required IN (0, 1)),
  sort_order INTEGER NOT NULL DEFAULT 0,
  enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
  default_value_json TEXT,
  suggestion_enabled INTEGER NOT NULL DEFAULT 0 CHECK(suggestion_enabled IN (0, 1)),
  is_system INTEGER NOT NULL DEFAULT 0 CHECK(is_system IN (0, 1)),
  ui_managed INTEGER NOT NULL DEFAULT 0 CHECK(ui_managed IN (0, 1)),
  version INTEGER NOT NULL DEFAULT 1 CHECK(version > 0),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(service_definition_id, parameter_key)
);

CREATE INDEX IF NOT EXISTS idx_service_parameters_definition ON service_parameters(service_definition_id, sort_order);

CREATE TABLE IF NOT EXISTS parameter_options (
  id TEXT PRIMARY KEY NOT NULL,
  parameter_id TEXT NOT NULL REFERENCES service_parameters(id) ON UPDATE CASCADE ON DELETE CASCADE,
  label TEXT NOT NULL,
  value TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(parameter_id, value)
);

CREATE INDEX IF NOT EXISTS idx_parameter_options_parameter ON parameter_options(parameter_id, sort_order);

CREATE TABLE IF NOT EXISTS pricing_rules (
  id TEXT PRIMARY KEY NOT NULL,
  service_definition_id TEXT NOT NULL REFERENCES service_definitions(id) ON UPDATE CASCADE ON DELETE CASCADE,
  parameter_id TEXT REFERENCES service_parameters(id) ON UPDATE CASCADE ON DELETE CASCADE,
  option_id TEXT REFERENCES parameter_options(id) ON UPDATE CASCADE ON DELETE CASCADE,
  quantity_parameter_id TEXT REFERENCES service_parameters(id) ON UPDATE CASCADE ON DELETE SET NULL,
  name TEXT NOT NULL,
  rule_type TEXT NOT NULL CHECK(rule_type IN ('fixed_amount','hours','per_unit','percentage','multiplier','external_cost')),
  numeric_value_micros INTEGER,
  amount_ars_minor INTEGER CHECK(amount_ars_minor IS NULL OR amount_ars_minor >= 0),
  amount_usd_minor INTEGER CHECK(amount_usd_minor IS NULL OR amount_usd_minor >= 0),
  sort_order INTEGER NOT NULL DEFAULT 0,
  enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
  version INTEGER NOT NULL DEFAULT 1 CHECK(version > 0),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_pricing_rules_definition ON pricing_rules(service_definition_id, sort_order);

CREATE TABLE IF NOT EXISTS economic_profiles (
  currency TEXT PRIMARY KEY NOT NULL CHECK(currency IN ('ARS', 'USD')),
  monthly_income_target_minor INTEGER CHECK(monthly_income_target_minor IS NULL OR monthly_income_target_minor >= 0),
  monthly_expenses_minor INTEGER CHECK(monthly_expenses_minor IS NULL OR monthly_expenses_minor >= 0),
  billable_hours_micros INTEGER CHECK(billable_hours_micros IS NULL OR billable_hours_micros > 0),
  reserve_tax_micros INTEGER CHECK(reserve_tax_micros IS NULL OR (reserve_tax_micros >= 0 AND reserve_tax_micros < 1000000)),
  desired_margin_micros INTEGER CHECK(desired_margin_micros IS NULL OR (desired_margin_micros >= 0 AND desired_margin_micros < 1000000)),
  default_urgency_micros INTEGER CHECK(default_urgency_micros IS NULL OR default_urgency_micros >= 0),
  work_days INTEGER CHECK(work_days IS NULL OR work_days > 0),
  vacation_weeks INTEGER CHECK(vacation_weeks IS NULL OR (vacation_weeks >= 0 AND vacation_weeks < 52)),
  manual_hourly_rate_minor INTEGER CHECK(manual_hourly_rate_minor IS NULL OR manual_hourly_rate_minor >= 0),
  updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO economic_profiles (currency, manual_hourly_rate_minor, updated_at)
SELECT 'ARS', hourly_rate_ars_minor, updated_at FROM app_settings WHERE id = 1;
INSERT OR IGNORE INTO economic_profiles (currency, manual_hourly_rate_minor, updated_at)
SELECT 'USD', hourly_rate_usd_minor, updated_at FROM app_settings WHERE id = 1;

CREATE TABLE IF NOT EXISTS market_sources (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  base_url TEXT,
  source_type TEXT NOT NULL CHECK(source_type IN ('freelance_marketplace','rate_benchmark','professional_tariff','salary','job_board','agency_pricing','methodology','currency','other')),
  regions_json TEXT NOT NULL DEFAULT '[]',
  supported_services_json TEXT NOT NULL DEFAULT '[]',
  priority INTEGER NOT NULL DEFAULT 0,
  enabled INTEGER NOT NULL DEFAULT 0 CHECK(enabled IN (0, 1)),
  usage_mode TEXT NOT NULL CHECK(usage_mode IN ('market_price','salary_context','rate_methodology','currency','context_only')),
  acquisition_mode TEXT NOT NULL DEFAULT 'disabled' CHECK(acquisition_mode IN ('auto_http','auto_browser','manual','disabled')),
  cooldown_hours INTEGER CHECK(cooldown_hours IS NULL OR cooldown_hours >= 0),
  notes TEXT,
  is_system_source INTEGER NOT NULL DEFAULT 0 CHECK(is_system_source IN (0, 1)),
  system_key TEXT UNIQUE,
  default_data_json TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO service_definitions (id, service_type, name, description, version, enabled, suggestions_enabled, default_strategy, created_at, updated_at) VALUES
('service-video-editing', 'video-editing', 'Edición de video', 'Módulo configurable de producción y edición audiovisual.', 1, 1, 1, 'balanced', strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('service-programming', 'programming', 'Programación', 'Módulo configurable para productos de software y automatización.', 1, 1, 1, 'balanced', strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now'));

INSERT OR IGNORE INTO service_parameters (id, service_definition_id, parameter_key, name, label, parameter_type, description, required, sort_order, enabled, suggestion_enabled, is_system, ui_managed, created_at, updated_at) VALUES
('video-piece-type','service-video-editing','pieceType','Tipo de pieza','Tipo de pieza','single_select','Formato principal de la pieza.',0,10,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-quantity','service-video-editing','quantity','Cantidad','Cantidad de piezas','number','Cantidad total de entregables.',1,20,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-raw','service-video-editing','rawMinutes','Material bruto','Material bruto','duration','Duración del material recibido.',0,30,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-duration','service-video-editing','finalDuration','Duración final','Duración final','duration','Duración del entregable final.',0,40,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-resolution','service-video-editing','resolution','Resolución','Resolución','single_select','Resolución de entrega.',1,50,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-level','service-video-editing','editingLevel','Nivel de edición','Nivel de edición','single_select','Complejidad editorial general.',0,60,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-revisions','service-video-editing','revisions','Revisiones','Revisiones incluidas','number',NULL,0,70,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-urgency','service-video-editing','urgency','Urgencia','Urgencia','single_select',NULL,0,80,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-formats','service-video-editing','formats','Formatos','Formatos','multi_select',NULL,0,90,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-hours','service-video-editing','estimatedHours','Horas base','Horas estimadas','number','Horas estimadas manuales antes de reglas.',0,100,1,1,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-color','service-video-editing','color','Color','Color','single_select',NULL,0,110,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-audio','service-video-editing','audio','Audio','Audio','single_select',NULL,0,120,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-subtitles','service-video-editing','subtitles','Subtítulos','Subtítulos','single_select',NULL,0,130,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-ai','service-video-editing','videoAi','Video IA','Video IA','single_select',NULL,0,140,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-voice-ai','service-video-editing','voiceAi','Voz IA','Voz IA','boolean',NULL,0,150,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-sound-ai','service-video-editing','soundAi','Sonido IA','Sonido IA','boolean',NULL,0,160,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-bg','service-video-editing','backgroundRemoval','Remoción de fondo','Remoción de fondo','boolean',NULL,0,170,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-motion','service-video-editing','motion','Motion','Motion','single_select',NULL,0,180,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-broll','service-video-editing','broll','B-roll','Material / B-roll','single_select',NULL,0,190,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('video-versions','service-video-editing','additionalVersions','Versiones adicionales','Versiones adicionales','number',NULL,0,200,1,0,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('program-hours','service-programming','estimatedHours','Horas base','Horas estimadas','number','Estimación manual inicial antes de reglas.',0,5,1,1,1,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('program-type','service-programming','projectType','Tipo de proyecto','Tipo de proyecto','single_select',NULL,0,10,1,0,1,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('program-frontend','service-programming','frontend','Frontend','Frontend','single_select',NULL,0,20,1,0,1,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('program-backend','service-programming','backend','Backend','Backend','single_select',NULL,0,30,1,0,1,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('program-db','service-programming','database','Base de datos','Base de datos','boolean',NULL,0,40,1,0,1,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('program-auth','service-programming','authentication','Autenticación','Autenticación','single_select',NULL,0,50,1,0,1,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('program-integrations','service-programming','integrations','Integraciones','Cantidad de integraciones','number',NULL,0,60,1,0,1,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('program-screens','service-programming','screens','Pantallas','Cantidad de pantallas','number',NULL,0,70,1,0,1,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('program-responsive','service-programming','responsive','Responsive','Responsive','single_select',NULL,0,80,1,0,1,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('program-deploy','service-programming','deploy','Deploy','Deploy','boolean',NULL,0,90,1,0,1,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('program-complexity','service-programming','complexity','Complejidad','Complejidad','single_select',NULL,0,100,1,0,1,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'));

INSERT OR IGNORE INTO parameter_options (id, parameter_id, label, value, sort_order, enabled, created_at, updated_at) VALUES
('opt-v-piece-reel','video-piece-type','Reel / Short','reel-short',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-v-piece-youtube','video-piece-type','YouTube','youtube',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-v-piece-ad','video-piece-type','Publicidad','advertising',30,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-v-piece-inst','video-piece-type','Institucional','institutional',40,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-v-piece-podcast','video-piece-type','Podcast','podcast',50,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-v-res-1080','video-resolution','Full HD 1080p','1080p',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-v-level-basic','video-level','Básica','basic',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-v-level-pro','video-level','Profesional','professional',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-v-level-adv','video-level','Avanzada','advanced',30,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-v-level-custom','video-level','Custom','custom',40,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-v-motion-none','video-motion','Ninguno','none',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-v-motion-basic','video-motion','Básico','basic',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-v-motion-ai','video-motion','Asistido por IA','ai-assisted',30,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-v-motion-custom','video-motion','Custom','custom',40,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-type-landing','program-type','Landing','landing',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-type-web','program-type','Web','web',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-type-desktop','program-type','Desktop App','desktop',30,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-type-dashboard','program-type','Dashboard','dashboard',40,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-type-internal','program-type','Herramienta interna','internal',50,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-type-auto','program-type','Automatización','automation',60,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-type-ai','program-type','IA','ai',70,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-type-custom','program-type','Custom','custom',80,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-front-no','program-frontend','No','none',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-front-basic','program-frontend','Básico','basic',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-front-mid','program-frontend','Intermedio','intermediate',30,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-front-premium','program-frontend','Premium','premium',40,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-back-no','program-backend','No','none',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-back-basic','program-backend','Básico','basic',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-back-complex','program-backend','Complejo','complex',30,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-auth-no','program-auth','No','none',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-auth-basic','program-auth','Básica','basic',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-auth-roles','program-auth','Roles / permisos','roles',30,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-responsive-desktop','program-responsive','Desktop','desktop',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-responsive-both','program-responsive','Desktop + Mobile','desktop-mobile',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-complex-low','program-complexity','Baja','low',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-complex-mid','program-complexity','Media','medium',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('opt-p-complex-high','program-complexity','Alta','high',30,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'));

INSERT OR IGNORE INTO market_sources (id,name,source_type,regions_json,supported_services_json,priority,enabled,usage_mode,acquisition_mode,is_system_source,system_key,default_data_json,created_at,updated_at) VALUES
('source-tarifario','Tarifario.org','professional_tariff','["AR","LATAM"]','["video-editing","programming"]',10,0,'market_price','disabled',1,'tarifario','{"enabled":false,"priority":10,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-adg','ADG','professional_tariff','["AR"]','["video-editing"]',20,0,'market_price','disabled',1,'adg','{"enabled":false,"priority":20,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-workana','Workana','freelance_marketplace','["LATAM"]','["video-editing","programming"]',30,0,'market_price','disabled',1,'workana','{"enabled":false,"priority":30,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-glassdoor','Glassdoor','salary','["GLOBAL"]','["video-editing","programming"]',40,0,'salary_context','disabled',1,'glassdoor','{"enabled":false,"priority":40,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-randstad','Randstad','salary','["AR","GLOBAL"]','["video-editing","programming"]',50,0,'salary_context','disabled',1,'randstad','{"enabled":false,"priority":50,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-michael-page','Michael Page','salary','["AR","GLOBAL"]','["video-editing","programming"]',60,0,'salary_context','disabled',1,'michael-page','{"enabled":false,"priority":60,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-remotejobs-lat','RemoteJobs.lat','job_board','["LATAM"]','["programming"]',70,0,'salary_context','disabled',1,'remotejobs-lat','{"enabled":false,"priority":70,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-bcra','BCRA','currency','["AR"]','["video-editing","programming"]',80,0,'currency','disabled',1,'bcra','{"enabled":false,"priority":80,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-upwork','Upwork','freelance_marketplace','["GLOBAL"]','["video-editing","programming"]',100,0,'market_price','disabled',1,'upwork','{"enabled":false,"priority":100,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-yunojuno','YunoJuno','freelance_marketplace','["GLOBAL"]','["video-editing","programming"]',110,0,'market_price','disabled',1,'yunojuno','{"enabled":false,"priority":110,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-twine','Twine','freelance_marketplace','["GLOBAL"]','["video-editing","programming"]',120,0,'market_price','disabled',1,'twine','{"enabled":false,"priority":120,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-malt','Malt','freelance_marketplace','["GLOBAL"]','["video-editing","programming"]',130,0,'market_price','disabled',1,'malt','{"enabled":false,"priority":130,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-contra','Contra','freelance_marketplace','["GLOBAL"]','["video-editing","programming"]',140,0,'market_price','disabled',1,'contra','{"enabled":false,"priority":140,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-freelancer','Freelancer.com','freelance_marketplace','["GLOBAL"]','["video-editing","programming"]',150,0,'market_price','disabled',1,'freelancer','{"enabled":false,"priority":150,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-peopleperhour','PeoplePerHour','freelance_marketplace','["GLOBAL"]','["video-editing","programming"]',160,0,'market_price','disabled',1,'peopleperhour','{"enabled":false,"priority":160,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-fiverr','Fiverr','freelance_marketplace','["GLOBAL"]','["video-editing","programming"]',170,0,'market_price','disabled',1,'fiverr','{"enabled":false,"priority":170,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-toptal','Toptal','freelance_marketplace','["GLOBAL"]','["programming"]',180,0,'market_price','disabled',1,'toptal','{"enabled":false,"priority":180,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-guru','Guru','freelance_marketplace','["GLOBAL"]','["video-editing","programming"]',190,0,'market_price','disabled',1,'guru','{"enabled":false,"priority":190,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-codeable','Codeable','freelance_marketplace','["GLOBAL"]','["programming"]',200,0,'market_price','disabled',1,'codeable','{"enabled":false,"priority":200,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-levels','Levels.fyi','salary','["GLOBAL"]','["programming"]',210,0,'salary_context','disabled',1,'levels','{"enabled":false,"priority":210,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-payscale','PayScale','salary','["GLOBAL"]','["programming"]',220,0,'salary_context','disabled',1,'payscale','{"enabled":false,"priority":220,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-indeed','Indeed Salaries','salary','["GLOBAL"]','["programming"]',230,0,'salary_context','disabled',1,'indeed','{"enabled":false,"priority":230,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-salary-com','Salary.com','salary','["GLOBAL"]','["programming"]',240,0,'salary_context','disabled',1,'salary-com','{"enabled":false,"priority":240,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-talent','Talent.com','salary','["GLOBAL"]','["programming"]',250,0,'salary_context','disabled',1,'talent','{"enabled":false,"priority":250,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-stackoverflow','Stack Overflow Developer Survey','rate_benchmark','["GLOBAL"]','["programming"]',260,0,'salary_context','disabled',1,'stackoverflow','{"enabled":false,"priority":260,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-ziprecruiter','ZipRecruiter Salaries','salary','["GLOBAL"]','["programming"]',270,0,'salary_context','disabled',1,'ziprecruiter','{"enabled":false,"priority":270,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-comparably','Comparably','salary','["GLOBAL"]','["programming"]',280,0,'salary_context','disabled',1,'comparably','{"enabled":false,"priority":280,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-salaryexpert','SalaryExpert','salary','["GLOBAL"]','["programming"]',290,0,'salary_context','disabled',1,'salaryexpert','{"enabled":false,"priority":290,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-paylab','Paylab','salary','["GLOBAL"]','["programming"]',300,0,'salary_context','disabled',1,'paylab','{"enabled":false,"priority":300,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-world-salaries','World Salaries','salary','["GLOBAL"]','["programming"]',310,0,'salary_context','disabled',1,'world-salaries','{"enabled":false,"priority":310,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-salary-explorer','Salary Explorer','salary','["GLOBAL"]','["programming"]',320,0,'salary_context','disabled',1,'salary-explorer','{"enabled":false,"priority":320,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-coroflot','Coroflot','rate_benchmark','["GLOBAL"]','["video-editing"]',330,0,'market_price','disabled',1,'coroflot','{"enabled":false,"priority":330,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-aiga','AIGA','professional_tariff','["GLOBAL"]','["video-editing"]',340,0,'rate_methodology','disabled',1,'aiga','{"enabled":false,"priority":340,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-99designs','99designs','freelance_marketplace','["GLOBAL"]','["video-editing"]',350,0,'market_price','disabled',1,'99designs','{"enabled":false,"priority":350,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-creativepool','Creativepool','freelance_marketplace','["GLOBAL"]','["video-editing"]',360,0,'market_price','disabled',1,'creativepool','{"enabled":false,"priority":360,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-clockify','Clockify','methodology','["GLOBAL"]','["video-editing","programming"]',370,0,'rate_methodology','disabled',1,'clockify','{"enabled":false,"priority":370,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-harvest','Harvest','methodology','["GLOBAL"]','["video-editing","programming"]',380,0,'rate_methodology','disabled',1,'harvest','{"enabled":false,"priority":380,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-bonsai','Bonsai','methodology','["GLOBAL"]','["video-editing","programming"]',390,0,'rate_methodology','disabled',1,'bonsai','{"enabled":false,"priority":390,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-pinebill','PineBill','methodology','["GLOBAL"]','["video-editing","programming"]',400,0,'rate_methodology','disabled',1,'pinebill','{"enabled":false,"priority":400,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-fastlancer','Fastlancer','methodology','["GLOBAL"]','["video-editing","programming"]',410,0,'rate_methodology','disabled',1,'fastlancer','{"enabled":false,"priority":410,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-timesheet','Timesheet.io','methodology','["GLOBAL"]','["video-editing","programming"]',420,0,'rate_methodology','disabled',1,'timesheet','{"enabled":false,"priority":420,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-frc','FreelanceRateCalculator.net','methodology','["GLOBAL"]','["video-editing","programming"]',430,0,'rate_methodology','disabled',1,'frc','{"enabled":false,"priority":430,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-fhr','FreelanceHourlyRate.com','methodology','["GLOBAL"]','["video-editing","programming"]',440,0,'rate_methodology','disabled',1,'fhr','{"enabled":false,"priority":440,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-nxcode','NxCode','methodology','["GLOBAL"]','["programming"]',450,0,'rate_methodology','disabled',1,'nxcode','{"enabled":false,"priority":450,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-truested','Truested','methodology','["GLOBAL"]','["video-editing","programming"]',460,0,'rate_methodology','disabled',1,'truested','{"enabled":false,"priority":460,"acquisitionMode":"disabled"}',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'));

