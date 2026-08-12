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
    pub notes: Option<String>,
    pub selected_price_kind: String,
    pub selected_price_minor: Option<i64>,
    pub floor_total_minor: Option<i64>,
    pub recommended_total_minor: Option<i64>,
    pub premium_total_minor: Option<i64>,
    pub snapshot_revision: i64,
    pub saved_at: Option<String>,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct QuoteHistoryItem {
    pub id: String,
    pub project_id: String,
    pub project_name: String,
    pub client_id: String,
    pub client_name: String,
    pub currency: String,
    pub status: String,
    pub notes: Option<String>,
    pub selected_price_kind: String,
    pub selected_price_minor: Option<i64>,
    pub floor_total_minor: Option<i64>,
    pub recommended_total_minor: Option<i64>,
    pub premium_total_minor: Option<i64>,
    pub snapshot_revision: i64,
    pub saved_at: String,
    pub updated_at: String,
    pub service_count: i64,
    pub service_titles: String,
    pub service_types: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct QuoteSnapshotRevision {
    pub revision: i64,
    pub reason: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteHistoryDetail {
    pub quote: QuoteHistoryItem,
    pub snapshot_json: String,
    pub snapshot_created_at: String,
    pub displayed_revision: i64,
    pub revisions: Vec<QuoteSnapshotRevision>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveQuoteSnapshotInput {
    pub quote_id: String,
    pub notes: Option<String>,
    pub selected_price_kind: String,
    pub selected_price_minor: Option<i64>,
    pub floor_total_minor: Option<i64>,
    pub recommended_total_minor: Option<i64>,
    pub premium_total_minor: Option<i64>,
    pub total_hours_micros: i64,
    pub external_costs_minor: i64,
    pub effective_hourly_minor: Option<i64>,
    pub margin_micros: Option<i64>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateQuoteAdminInput {
    pub quote_id: String,
    pub project_name: String,
    pub client_id: String,
    pub notes: Option<String>,
    pub status: String,
    pub selected_price_kind: String,
    pub selected_price_minor: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateQuoteInput {
    pub quote_id: String,
    pub project_name: Option<String>,
    pub client_id: Option<String>,
    pub revision: Option<i64>,
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
    pub help_mode: String,
    pub local_ai_enabled: bool,
    pub ollama_base_url: String,
    pub ollama_model: Option<String>,
    pub ai_auto_apply_high_confidence: bool,
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
    pub engine_id: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
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
    pub purpose: Option<String>,
    pub data_contribution: Option<String>,
    pub app_benefit: Option<String>,
    pub participates_in_suggestions: bool,
    pub automation_status: String,
    pub current_status: String,
    pub adapter_key: Option<String>,
    pub last_request_at: Option<String>,
    pub last_success_at: Option<String>,
    pub last_failure_at: Option<String>,
    pub cooldown_until: Option<String>,
    pub consecutive_failures: i64,
    pub last_http_status: Option<i64>,
    pub last_error: Option<String>,
    pub observation_count: i64,
    pub archived_at: Option<String>,
    pub business_source_type: String,
    pub market_country: Option<String>,
    pub source_currency: Option<String>,
    pub source_updated_at: Option<String>,
    pub classification_origin: String,
    pub classification_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct EngineCategory {
    pub id: String,
    pub parent_id: Option<String>,
    pub slug: String,
    pub name: String,
    pub engine_type: Option<String>,
    pub description: Option<String>,
    pub is_system: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PricingEngine {
    pub id: String,
    pub engine_key: String,
    pub name: String,
    pub description: Option<String>,
    pub engine_type: String,
    pub category_id: Option<String>,
    pub calculator_key: String,
    pub service_definition_id: Option<String>,
    pub unit_kind: String,
    pub tags_json: String,
    pub status: String,
    pub classification_origin: String,
    pub classification_confidence_micros: Option<i64>,
    pub classification_explanation: Option<String>,
    pub classification_version: i64,
    pub is_system: bool,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PricingEngineSource {
    pub engine_id: String,
    pub source_id: String,
    pub role: String,
    pub preference: String,
    pub participates_in_suggestions: bool,
    pub match_score_micros: i64,
    pub explanation: Option<String>,
    pub assigned_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationProposal {
    pub engine_type: String,
    pub category_id: Option<String>,
    pub category_path: Vec<String>,
    pub calculator_key: String,
    pub business_activity: String,
    pub pricing_units: Vec<String>,
    pub suggested_cost_types: Vec<String>,
    pub suggested_source_types: Vec<String>,
    pub tags: Vec<String>,
    pub confidence: f64,
    pub explanation: String,
    pub clarification_question: Option<String>,
    pub ai_assisted: bool,
    pub ai_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaModel {
    pub name: String,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
    pub size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaStatus {
    pub available: bool,
    pub base_url: String,
    pub selected_model: Option<String>,
    pub models: Vec<OllamaModel>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MarketObservation {
    pub id: String,
    pub source_id: String,
    pub source_name: String,
    pub origin: String,
    pub service_type: String,
    pub subservice: Option<String>,
    pub category: Option<String>,
    pub region: String,
    pub country: Option<String>,
    pub currency: String,
    pub price_type: String,
    pub unit: String,
    pub price_min_minor: Option<i64>,
    pub price_max_minor: Option<i64>,
    pub price_value_minor: Option<i64>,
    pub original_value_text: String,
    pub converted_value_minor: Option<i64>,
    pub converted_currency: Option<String>,
    pub exchange_rate_micros: Option<i64>,
    pub exchange_rate_date: Option<String>,
    pub exchange_rate_source: Option<String>,
    pub experience_level: Option<String>,
    pub client_tier: Option<String>,
    pub source_type: String,
    pub source_url: String,
    pub published_at: Option<String>,
    pub retrieved_at: String,
    pub parser_version: String,
    pub confidence: String,
    pub comparison_eligibility: String,
    pub exclusion_reason: Option<String>,
    pub raw_fingerprint: String,
    pub evidence_snippet: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub snapshot_included: Option<bool>,
    pub snapshot_exclusion_reason: Option<String>,
    pub snapshot_normalized_value_minor: Option<i64>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MarketSnapshot {
    pub id: String,
    pub quote_id: Option<String>,
    pub quote_service_id: Option<String>,
    pub query_context_json: String,
    pub currency: String,
    pub observation_count: i64,
    pub comparable_observation_count: i64,
    pub source_count: i64,
    pub minimum_filtered_minor: Option<i64>,
    pub p25_minor: Option<i64>,
    pub market_median_minor: Option<i64>,
    pub p75_minor: Option<i64>,
    pub maximum_filtered_minor: Option<i64>,
    pub confidence_level: String,
    pub calculated_price_minor: Option<i64>,
    pub suggested_price_minor: Option<i64>,
    pub final_price_minor_at_creation: Option<i64>,
    pub base_service_revision: Option<i64>,
    pub suggestion_update_status: String,
    pub suggestion_update_message: Option<String>,
    pub summary_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketOverview {
    pub latest_snapshot: Option<MarketSnapshot>,
    pub observations: Vec<MarketObservation>,
    pub history: Vec<MarketSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketResearchJobItem {
    pub source_id: String,
    pub source_name: String,
    pub status: String,
    pub message: Option<String>,
    pub observation_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketResearchJob {
    pub id: String,
    pub quote_service_id: String,
    pub base_service_revision: i64,
    pub status: String,
    pub completed: i64,
    pub total: i64,
    pub cancel_requested: bool,
    pub items: Vec<MarketResearchJobItem>,
    pub snapshot_id: Option<String>,
    pub suggestion_update_status: String,
    pub suggestion_update_message: Option<String>,
    pub error: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
    #[serde(skip)]
    pub baseline: MarketResearchBaseline,
}

#[derive(Debug, Clone, Default)]
pub struct MarketResearchBaseline {
    pub quote_id: String,
    pub service_type: String,
    pub configuration_json: String,
    pub calculated_price_minor: Option<i64>,
    pub final_price_minor: Option<i64>,
    pub has_override: bool,
    pub currency: String,
    pub market_scope: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceTestResult {
    pub source_id: String,
    pub status: String,
    pub message: String,
    pub http_status: Option<i64>,
    pub observations: Vec<MarketObservationPreview>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketObservationPreview {
    pub service_type: String,
    pub subservice: Option<String>,
    pub price_min_minor: Option<i64>,
    pub price_max_minor: Option<i64>,
    pub price_value_minor: Option<i64>,
    pub currency: String,
    pub unit: String,
    pub price_type: String,
    pub region: String,
    pub evidence: Option<String>,
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
    pub engine_categories: Vec<EngineCategory>,
    pub pricing_engines: Vec<PricingEngine>,
    pub engine_sources: Vec<PricingEngineSource>,
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
    pub help_mode: String,
    pub local_ai_enabled: bool,
    pub ollama_base_url: String,
    pub ollama_model: Option<String>,
    pub ai_auto_apply_high_confidence: bool,
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
    pub engine_id: String,
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
    pub purpose: Option<String>,
    pub data_contribution: Option<String>,
    pub app_benefit: Option<String>,
    pub participates_in_suggestions: bool,
    pub business_source_type: Option<String>,
    pub market_country: Option<String>,
    pub source_currency: Option<String>,
    pub source_updated_at: Option<String>,
    pub classification_origin: Option<String>,
    pub classification_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationInput {
    pub name: String,
    pub description: Option<String>,
    pub deliverable_kind: Option<String>,
    pub activity_kind: Option<String>,
    pub pricing_unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceClassificationInput {
    pub name: String,
    pub base_url: Option<String>,
    pub purpose: Option<String>,
    pub data_contribution: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceClassificationProposal {
    pub business_source_type: String,
    pub suggested_engine_types: Vec<String>,
    pub role: String,
    pub tags: Vec<String>,
    pub confidence: f64,
    pub explanation: String,
    pub ai_assisted: bool,
    pub ai_error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PricingEngineInput {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub engine_type: String,
    pub category_id: Option<String>,
    pub calculator_key: String,
    pub unit_kind: String,
    pub tags: Vec<String>,
    pub status: String,
    pub classification_origin: String,
    pub classification_confidence: Option<f64>,
    pub classification_explanation: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineSourceInput {
    pub engine_id: String,
    pub source_id: String,
    pub role: String,
    pub preference: String,
    pub participates_in_suggestions: bool,
    pub match_score: f64,
    pub explanation: Option<String>,
    pub assigned_by: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualObservationInput {
    pub source_id: String,
    pub service_type: String,
    pub subservice: Option<String>,
    pub category: Option<String>,
    pub region: String,
    pub country: Option<String>,
    pub currency: String,
    pub price_type: String,
    pub unit: String,
    pub price_min_minor: Option<i64>,
    pub price_max_minor: Option<i64>,
    pub price_value_minor: Option<i64>,
    pub experience_level: Option<String>,
    pub client_tier: Option<String>,
    pub published_at: Option<String>,
    pub source_url: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MarketObservationFilter {
    pub service_type: Option<String>,
    pub region: Option<String>,
    pub source_id: Option<String>,
    pub price_type: Option<String>,
    pub currency: Option<String>,
    pub query: Option<String>,
}
