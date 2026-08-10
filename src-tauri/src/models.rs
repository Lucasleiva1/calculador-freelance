use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    pub id: String,
    pub name: String,
    pub company: Option<String>,
    pub email: Option<String>,
    pub whatsapp: Option<String>,
    pub country: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub client_id: String,
    pub client_name: String,
    pub name: String,
    pub currency: String,
    pub market_scope: Option<String>,
    pub status: String,
    pub total_minor: Option<i64>,
    pub unpriced_count: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub id: String,
    pub project_id: String,
    pub version: i64,
    pub status: String,
    pub currency: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct QuoteService {
    pub id: String,
    pub quote_id: String,
    pub service_type: String,
    pub title: String,
    pub sort_order: i64,
    pub configuration_version: i64,
    pub configuration_json: String,
    pub calculated_subtotal_minor: Option<i64>,
    pub suggested_subtotal_minor: Option<i64>,
    pub final_subtotal_minor: Option<i64>,
    pub has_override: bool,
    pub manual_subtotal_minor: Option<i64>,
    pub manual_reason: Option<String>,
    pub pricing_snapshot_json: Option<String>,
    pub service_definition_version: Option<i64>,
    pub row_revision: i64,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub id: String,
    pub service_type: String,
    pub name: String,
    pub origin: String,
    pub system_key: Option<String>,
    pub configuration_version: i64,
    pub definition_version: i64,
    pub configuration_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String,
    pub hourly_rate_ars_minor: Option<i64>,
    pub hourly_rate_usd_minor: Option<i64>,
    pub usd_to_ars_micros: Option<i64>,
    pub active_project_id: Option<String>,
    pub suggestions_enabled: bool,
    pub suggestion_strategy: String,
    pub base_currency: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDefinition {
    pub id: String,
    pub service_type: String,
    pub name: String,
    pub description: Option<String>,
    pub version: i64,
    pub enabled: bool,
    pub suggestions_enabled: bool,
    pub default_strategy: String,
    pub competitive_margin_micros: Option<i64>,
    pub balanced_margin_micros: Option<i64>,
    pub premium_margin_micros: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ServiceParameter {
    pub id: String,
    pub service_definition_id: String,
    pub parameter_key: String,
    pub name: String,
    pub label: String,
    pub parameter_type: String,
    pub description: Option<String>,
    pub required: bool,
    pub sort_order: i64,
    pub enabled: bool,
    pub default_value_json: Option<String>,
    pub suggestion_enabled: bool,
    pub is_system: bool,
    pub ui_managed: bool,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ParameterOption {
    pub id: String,
    pub parameter_id: String,
    pub label: String,
    pub value: String,
    pub sort_order: i64,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PricingRule {
    pub id: String,
    pub service_definition_id: String,
    pub parameter_id: Option<String>,
    pub option_id: Option<String>,
    pub quantity_parameter_id: Option<String>,
    pub name: String,
    pub rule_type: String,
    pub numeric_value_micros: Option<i64>,
    pub amount_ars_minor: Option<i64>,
    pub amount_usd_minor: Option<i64>,
    pub sort_order: i64,
    pub enabled: bool,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct EconomicProfile {
    pub currency: String,
    pub monthly_income_target_minor: Option<i64>,
    pub monthly_expenses_minor: Option<i64>,
    pub billable_hours_micros: Option<i64>,
    pub reserve_tax_micros: Option<i64>,
    pub desired_margin_micros: Option<i64>,
    pub default_urgency_micros: Option<i64>,
    pub work_days: Option<i64>,
    pub vacation_weeks: Option<i64>,
    pub manual_hourly_rate_minor: Option<i64>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MarketSource {
    pub id: String,
    pub name: String,
    pub base_url: Option<String>,
    pub source_type: String,
    pub regions_json: String,
    pub supported_services_json: String,
    pub priority: i64,
    pub enabled: bool,
    pub usage_mode: String,
    pub acquisition_mode: String,
    pub cooldown_hours: Option<i64>,
    pub notes: Option<String>,
    pub is_system_source: bool,
    pub system_key: Option<String>,
    pub default_data_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PricingConfiguration {
    pub definitions: Vec<ServiceDefinition>,
    pub parameters: Vec<ServiceParameter>,
    pub options: Vec<ParameterOption>,
    pub rules: Vec<PricingRule>,
    pub economic_profiles: Vec<EconomicProfile>,
    pub market_sources: Vec<MarketSource>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub clients: Vec<Client>,
    pub projects: Vec<ProjectSummary>,
    pub presets: Vec<Preset>,
    pub settings: AppSettings,
    pub pricing: PricingConfiguration,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub project: ProjectSummary,
    pub quote: Quote,
    pub services: Vec<QuoteService>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInput {
    pub id: Option<String>,
    pub name: String,
    pub company: Option<String>,
    pub email: Option<String>,
    pub whatsapp: Option<String>,
    pub country: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectInput {
    pub name: String,
    pub client_id: Option<String>,
    pub new_client: Option<ClientInput>,
    pub currency: String,
    pub market_scope: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveServiceInput {
    pub id: String,
    pub title: String,
    pub configuration_version: i64,
    pub configuration_json: String,
    pub calculated_subtotal_minor: Option<i64>,
    pub suggested_subtotal_minor: Option<i64>,
    pub final_subtotal_minor: Option<i64>,
    pub has_override: bool,
    pub manual_subtotal_minor: Option<i64>,
    pub manual_reason: Option<String>,
    pub pricing_snapshot_json: Option<String>,
    pub service_definition_version: Option<i64>,
    pub expected_revision: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInput {
    pub theme: String,
    pub hourly_rate_ars_minor: Option<i64>,
    pub hourly_rate_usd_minor: Option<i64>,
    pub usd_to_ars_micros: Option<i64>,
    pub suggestions_enabled: bool,
    pub suggestion_strategy: String,
    pub base_currency: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetInput {
    pub id: Option<String>,
    pub service_type: String,
    pub name: String,
    pub configuration_version: i64,
    pub definition_version: Option<i64>,
    pub configuration_json: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDefinitionInput {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub suggestions_enabled: bool,
    pub default_strategy: String,
    pub competitive_margin_micros: Option<i64>,
    pub balanced_margin_micros: Option<i64>,
    pub premium_margin_micros: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceParameterInput {
    pub id: Option<String>,
    pub service_definition_id: String,
    pub parameter_key: String,
    pub name: String,
    pub label: String,
    pub parameter_type: String,
    pub description: Option<String>,
    pub required: bool,
    pub sort_order: i64,
    pub enabled: bool,
    pub default_value_json: Option<String>,
    pub suggestion_enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterOptionInput {
    pub id: Option<String>,
    pub parameter_id: String,
    pub label: String,
    pub value: String,
    pub sort_order: i64,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PricingRuleInput {
    pub id: Option<String>,
    pub service_definition_id: String,
    pub parameter_id: Option<String>,
    pub option_id: Option<String>,
    pub quantity_parameter_id: Option<String>,
    pub name: String,
    pub rule_type: String,
    pub numeric_value_micros: Option<i64>,
    pub amount_ars_minor: Option<i64>,
    pub amount_usd_minor: Option<i64>,
    pub sort_order: i64,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EconomicProfileInput {
    pub currency: String,
    pub monthly_income_target_minor: Option<i64>,
    pub monthly_expenses_minor: Option<i64>,
    pub billable_hours_micros: Option<i64>,
    pub reserve_tax_micros: Option<i64>,
    pub desired_margin_micros: Option<i64>,
    pub default_urgency_micros: Option<i64>,
    pub work_days: Option<i64>,
    pub vacation_weeks: Option<i64>,
    pub manual_hourly_rate_minor: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSourceInput {
    pub id: Option<String>,
    pub name: String,
    pub base_url: Option<String>,
    pub source_type: String,
    pub regions_json: String,
    pub supported_services_json: String,
    pub priority: i64,
    pub enabled: bool,
    pub usage_mode: String,
    pub acquisition_mode: String,
    pub cooldown_hours: Option<i64>,
    pub notes: Option<String>,
}
