PRAGMA foreign_keys = ON;

-- La economía manual pertenece a una profesión/motor y a una moneda. Un motor
-- nuevo comienza sin perfil para impedir que herede silenciosamente una tarifa
-- de otra actividad.
CREATE TABLE IF NOT EXISTS engine_economic_profiles (
  engine_id TEXT NOT NULL REFERENCES pricing_engines(id) ON UPDATE CASCADE ON DELETE CASCADE,
  currency TEXT NOT NULL CHECK(currency IN ('ARS', 'USD')),
  monthly_income_target_minor INTEGER CHECK(monthly_income_target_minor IS NULL OR monthly_income_target_minor >= 0),
  monthly_expenses_minor INTEGER CHECK(monthly_expenses_minor IS NULL OR monthly_expenses_minor >= 0),
  billable_hours_micros INTEGER CHECK(billable_hours_micros IS NULL OR billable_hours_micros > 0),
  reserve_tax_micros INTEGER CHECK(reserve_tax_micros IS NULL OR (reserve_tax_micros >= 0 AND reserve_tax_micros < 1000000)),
  desired_margin_micros INTEGER CHECK(desired_margin_micros IS NULL OR (desired_margin_micros >= 0 AND desired_margin_micros < 1000000)),
  default_urgency_micros INTEGER CHECK(default_urgency_micros IS NULL OR default_urgency_micros >= 0),
  work_days INTEGER CHECK(work_days IS NULL OR work_days > 0),
  vacation_weeks INTEGER CHECK(vacation_weeks IS NULL OR (vacation_weeks >= 0 AND vacation_weeks < 52)),
  manual_hourly_rate_minor INTEGER CHECK(manual_hourly_rate_minor IS NULL OR manual_hourly_rate_minor >= 0),
  updated_at TEXT NOT NULL,
  PRIMARY KEY(engine_id, currency)
);

CREATE INDEX IF NOT EXISTS idx_engine_economic_profiles_currency
  ON engine_economic_profiles(currency, engine_id);

-- Los datos históricos fueron cargados mientras la pantalla activa era la de
-- edición de video. Se conservan únicamente allí; programación y los motores
-- futuros quedan pendientes hasta que la persona complete sus propios datos.
INSERT OR IGNORE INTO engine_economic_profiles (
  engine_id,currency,monthly_income_target_minor,monthly_expenses_minor,
  billable_hours_micros,reserve_tax_micros,desired_margin_micros,
  default_urgency_micros,work_days,vacation_weeks,manual_hourly_rate_minor,updated_at
)
SELECT
  'engine-video-editing',currency,monthly_income_target_minor,monthly_expenses_minor,
  billable_hours_micros,reserve_tax_micros,desired_margin_micros,
  default_urgency_micros,work_days,vacation_weeks,manual_hourly_rate_minor,updated_at
FROM economic_profiles
WHERE EXISTS (SELECT 1 FROM pricing_engines WHERE id='engine-video-editing');
