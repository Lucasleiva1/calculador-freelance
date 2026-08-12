use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketQueryContext {
    pub service: String,
    pub subtype: Option<String>,
    pub region_targets: Vec<String>,
    pub level: Option<String>,
    pub duration_minutes: Option<f64>,
    pub quantity: Option<f64>,
    pub estimated_hours: Option<f64>,
    pub features: Vec<String>,
    pub client_tier: Option<String>,
    pub work_class: Option<String>,
}

impl MarketQueryContext {
    pub fn generic(service: String, regions: Vec<String>) -> Self {
        Self {
            service,
            subtype: None,
            region_targets: regions,
            level: None,
            duration_minutes: None,
            quantity: None,
            estimated_hours: None,
            features: Vec::new(),
            client_tier: None,
            work_class: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObservationDraft {
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
    pub experience_level: Option<String>,
    pub client_tier: Option<String>,
    pub source_url: String,
    pub published_at: Option<String>,
    pub confidence: String,
    pub comparison_eligibility: String,
    pub exclusion_reason: Option<String>,
    pub evidence_snippet: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug)]
pub struct AcquisitionResponse {
    pub body: String,
    pub http_status: u16,
    pub final_url: String,
    pub retry_after_seconds: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ComparableObservation {
    pub observation_id: String,
    pub source_id: String,
    pub normalized_value_minor: Option<i64>,
    pub included: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonSummary {
    pub minimum_filtered_minor: Option<i64>,
    pub p25_minor: Option<i64>,
    pub median_minor: Option<i64>,
    pub p75_minor: Option<i64>,
    pub maximum_filtered_minor: Option<i64>,
    pub confidence_level: String,
    pub comparable_count: i64,
    pub source_count: i64,
    pub recent_count: i64,
    pub salary_excluded_count: i64,
    pub explanations: Vec<String>,
}
