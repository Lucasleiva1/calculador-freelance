use std::{collections::BTreeSet, time::Duration};

use chrono::{SecondsFormat, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{Sqlite, SqlitePool, Transaction};
use tauri::State;
use url::Url;
use uuid::Uuid;

use crate::{
    db::AppState,
    error::{command_error, AppError, AppResult},
    models::{
        ClassificationInput, ClassificationProposal, EngineCategory, EngineSourceInput,
        OllamaModel, OllamaStatus, PricingEngine, PricingEngineInput, PricingEngineSource,
        SourceClassificationInput, SourceClassificationProposal,
    },
};

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn clean(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|character| match character {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            character if character.is_alphanumeric() || character.is_whitespace() => character,
            _ => ' ',
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn contains_any(text: &str, terms: &[&str]) -> bool {
    terms.iter().any(|term| text.contains(term))
}

fn unique(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn automatic_classification(input: &ClassificationInput) -> ClassificationProposal {
    let description = input.description.as_deref().unwrap_or_default();
    let text = normalize(&format!("{} {}", input.name, description));
    let deliverable = input
        .deliverable_kind
        .as_deref()
        .map(normalize)
        .unwrap_or_default();
    let activity = input
        .activity_kind
        .as_deref()
        .map(normalize)
        .unwrap_or_default();

    let hybrid_text = contains_any(
        &text,
        &[
            "diseno y produccion",
            "diseno y fabricacion",
            "servicio y producto",
            "incluye diseno y",
            "gestion y produccion",
        ],
    );
    let product_text = contains_any(
        &text,
        &[
            "venta de",
            "vendo ",
            "vender ",
            "fabricacion",
            "produccion de",
            "remera",
            "camiseta",
            "media",
            "empanada",
            "termo",
            "producto fisico",
            "comida",
            "packaging fisico",
        ],
    );
    let service_text = contains_any(
        &text,
        &[
            "edicion",
            "programacion",
            "software",
            "fotografia",
            "diseno",
            "consultoria",
            "community management",
            "motion graphics",
        ],
    );

    let engine_type = if deliverable == "both"
        || deliverable == "ambos"
        || activity == "both"
        || activity == "ambos"
        || hybrid_text
        || (product_text && service_text && contains_any(&text, &["diseno", "gestion"]))
    {
        "hybrid"
    } else if deliverable == "physical"
        || deliverable == "fisico"
        || activity == "sale"
        || activity == "venta"
        || product_text
    {
        "product"
    } else {
        "service"
    };

    let (category_id, category_path, mut tags) = if engine_type == "hybrid" {
        (
            Some("category-design-production".to_string()),
            vec!["Híbridos".to_string(), "Diseño y producción".to_string()],
            vec!["servicio".to_string(), "producto".to_string()],
        )
    } else if contains_any(
        &text,
        &[
            "empanada",
            "comida",
            "alimento",
            "panaderia",
            "bebida",
            "pizza",
        ],
    ) {
        (
            Some("category-food".to_string()),
            vec!["Productos".to_string(), "Alimentos".to_string()],
            vec!["alimentos".to_string(), "comidas preparadas".to_string()],
        )
    } else if contains_any(
        &text,
        &[
            "remera",
            "camiseta",
            "media",
            "indumentaria",
            "prenda",
            "textil",
        ],
    ) {
        (
            Some("category-apparel".to_string()),
            vec!["Productos".to_string(), "Indumentaria".to_string()],
            vec!["indumentaria".to_string(), "textil".to_string()],
        )
    } else if contains_any(&text, &["termo", "taza", "hogar", "objeto"]) {
        (
            Some("category-home".to_string()),
            vec!["Productos".to_string(), "Hogar y objetos".to_string()],
            vec!["hogar".to_string(), "objetos".to_string()],
        )
    } else if contains_any(&text, &["video", "fotografia", "motion", "audiovisual"]) {
        (
            Some("category-audiovisual".to_string()),
            vec!["Servicios".to_string(), "Audiovisual".to_string()],
            vec!["audiovisual".to_string()],
        )
    } else if contains_any(
        &text,
        &["programacion", "software", "web", "automatizacion"],
    ) {
        (
            Some("category-technology".to_string()),
            vec!["Servicios".to_string(), "Tecnología".to_string()],
            vec!["tecnología".to_string(), "digital".to_string()],
        )
    } else if contains_any(&text, &["diseno", "grafico", "estampa", "branding"]) {
        (
            Some("category-design".to_string()),
            vec!["Servicios".to_string(), "Diseño".to_string()],
            vec!["diseño".to_string()],
        )
    } else if engine_type == "product" {
        (
            Some("category-products".to_string()),
            vec!["Productos".to_string()],
            vec!["producto".to_string()],
        )
    } else {
        (
            Some("category-services".to_string()),
            vec!["Servicios".to_string()],
            vec!["servicio".to_string()],
        )
    };

    for term in [
        "remeras",
        "remera",
        "medias",
        "empanadas",
        "empanada",
        "termos",
        "termo",
    ] {
        if text.contains(term) {
            tags.push(term.trim_end_matches('s').to_string());
        }
    }

    let pricing_units = if let Some(unit) = clean(input.pricing_unit.clone()) {
        vec![unit]
    } else if engine_type == "product" {
        vec!["unidad".into(), "lote".into()]
    } else if engine_type == "hybrid" {
        vec!["proyecto".into(), "unidad".into()]
    } else {
        vec!["proyecto".into(), "hora".into()]
    };
    let (costs, sources, activity_label) = match engine_type {
        "product" => (
            vec![
                "materiales o mercadería".into(),
                "producción".into(),
                "packaging".into(),
                "costos operativos".into(),
                "merma".into(),
                "comisiones".into(),
                "impuestos".into(),
                "logística".into(),
            ],
            vec![
                "proveedores".into(),
                "costos internos".into(),
                "competidores".into(),
                "plataformas de venta".into(),
            ],
            "Producción o venta de un producto",
        ),
        "hybrid" => (
            vec![
                "horas profesionales".into(),
                "materiales".into(),
                "producción".into(),
                "gestión".into(),
                "logística".into(),
            ],
            vec![
                "tarifas profesionales".into(),
                "proveedores".into(),
                "costos internos".into(),
                "mercado comparable".into(),
            ],
            "Servicio profesional combinado con un producto",
        ),
        _ => (
            vec![
                "horas profesionales".into(),
                "costos directos".into(),
                "revisiones".into(),
                "urgencia".into(),
            ],
            vec![
                "tarifas profesionales".into(),
                "mercado freelance".into(),
                "costos internos".into(),
            ],
            "Prestación de un servicio profesional",
        ),
    };

    let structured = !deliverable.is_empty() || !activity.is_empty();
    let recognized = product_text || service_text || hybrid_text;
    let confidence: f64 = if structured && recognized {
        0.96
    } else if structured || recognized {
        0.86
    } else {
        0.58
    };
    let clarification_question = (confidence < 0.7).then_some(
        "¿Entregás un producto físico, un servicio profesional o ambas cosas?".to_string(),
    );
    let calculator_key = match engine_type {
        "product" => "physical-product-v1",
        "hybrid" => "hybrid-v1",
        _ => "professional-service-v1",
    };

    ClassificationProposal {
        engine_type: engine_type.to_string(),
        category_id,
        category_path,
        calculator_key: calculator_key.to_string(),
        business_activity: activity_label.to_string(),
        pricing_units,
        suggested_cost_types: costs,
        suggested_source_types: sources,
        tags: unique(tags),
        confidence,
        explanation: format!(
            "La clasificación automática detectó {} y la ubicó en {}.",
            activity_label.to_lowercase(),
            match engine_type {
                "product" => "Productos",
                "hybrid" => "Híbridos",
                _ => "Servicios",
            }
        ),
        clarification_question,
        ai_assisted: false,
        ai_error: None,
    }
}

pub fn automatic_source_classification(
    input: &SourceClassificationInput,
) -> SourceClassificationProposal {
    let text = normalize(&format!(
        "{} {} {} {} {}",
        input.name,
        input.base_url.as_deref().unwrap_or_default(),
        input.purpose.as_deref().unwrap_or_default(),
        input.data_contribution.as_deref().unwrap_or_default(),
        input.notes.as_deref().unwrap_or_default(),
    ));

    let supplier = contains_any(
        &text,
        &[
            "proveedor",
            "mayorista",
            "fabricante",
            "distribuidor",
            "insumo",
            "materia prima",
            "prenda base",
            "costo de estampado",
            "packaging",
        ],
    );
    let internal = contains_any(
        &text,
        &[
            "costo interno",
            "dato propio",
            "mis costos",
            "carga manual",
            "precio propio",
        ],
    );
    let competitor = contains_any(
        &text,
        &[
            "competidor",
            "competencia",
            "marca comparable",
            "precio comparable",
        ],
    );
    let platform = contains_any(
        &text,
        &[
            "marketplace",
            "mercado libre",
            "mercadolibre",
            "amazon",
            "etsy",
            "workana",
            "fiverr",
            "upwork",
            "plataforma",
        ],
    );
    let market = contains_any(
        &text,
        &[
            "tarifario",
            "benchmark",
            "mercado",
            "indice de precios",
            "referencia de precios",
        ],
    );

    let business_source_type = if supplier {
        "supplier"
    } else if internal {
        "internal"
    } else if competitor {
        "competitor"
    } else if platform {
        "platform"
    } else if market {
        "market"
    } else {
        "other"
    };

    let product_signal = contains_any(
        &text,
        &[
            "remera",
            "camiseta",
            "media",
            "empanada",
            "termo",
            "producto",
            "prenda",
            "estampado",
            "packaging",
            "logistica",
            "insumo",
            "unidad fisica",
        ],
    );
    let service_signal = contains_any(
        &text,
        &[
            "freelance",
            "tarifa profesional",
            "hora profesional",
            "programacion",
            "edicion",
            "video",
            "diseno",
            "fotografia",
            "salario",
            "honorario",
        ],
    );
    let suggested_engine_types = match (product_signal, service_signal) {
        (true, true) => vec!["product".into(), "hybrid".into(), "service".into()],
        (true, false) => vec!["product".into(), "hybrid".into()],
        (false, true) => vec!["service".into(), "hybrid".into()],
        (false, false) if supplier => vec!["product".into(), "hybrid".into()],
        (false, false) => vec!["service".into(), "product".into(), "hybrid".into()],
    };
    let role = match business_source_type {
        "supplier" | "internal" => "cost_input",
        "market" | "competitor" | "platform" => "reference",
        _ => "context",
    };
    let mut tags = Vec::new();
    for (term, tag) in [
        ("remera", "remeras"),
        ("camiseta", "indumentaria"),
        ("estampado", "estampado"),
        ("video", "video"),
        ("programacion", "programación"),
        ("fotografia", "fotografía"),
        ("empanada", "alimentos"),
        ("termo", "hogar"),
        ("packaging", "packaging"),
    ] {
        if text.contains(term) {
            tags.push(tag.to_string());
        }
    }
    tags.push(business_source_type.to_string());
    let confidence = if business_source_type == "other" {
        0.56
    } else {
        0.88
    };
    let explanation = match business_source_type {
        "supplier" => "Parece un proveedor: conviene usarlo como origen de costos directos, nunca como precio final.",
        "internal" => "Parece un dato propio: sirve como costo controlado por vos y queda separado del mercado.",
        "competitor" => "Parece una referencia de competidores: sirve para contraste comercial, no para decidir la fórmula.",
        "platform" => "Parece una plataforma: puede aportar precios publicados, comisiones o contexto comercial.",
        "market" => "Parece una fuente de mercado o tarifario: aporta referencias externas comparables.",
        _ => "No hay señales suficientes para una categoría específica; se conserva como contexto hasta que la corrijas.",
    };

    SourceClassificationProposal {
        business_source_type: business_source_type.into(),
        suggested_engine_types,
        role: role.into(),
        tags: unique(tags),
        confidence,
        explanation: explanation.into(),
        ai_assisted: false,
        ai_error: None,
    }
}

async fn classification_with_aliases(
    pool: &SqlitePool,
    input: &ClassificationInput,
) -> AppResult<ClassificationProposal> {
    let mut proposal = automatic_classification(input);
    let searchable = normalize(&format!(
        "{} {}",
        input.name,
        input.description.as_deref().unwrap_or_default()
    ));
    let aliases: Vec<(String, String, Option<String>, String)> = sqlx::query_as(
        "SELECT normalized_term,engine_type,category_id,tags_json
         FROM classification_aliases ORDER BY length(normalized_term) DESC,use_count DESC",
    )
    .fetch_all(pool)
    .await?;
    let structured_hint = !matches!(input.deliverable_kind.as_deref(), None | Some("unknown"))
        || !matches!(input.activity_kind.as_deref(), None | Some("unknown"));
    if let Some((term, engine_type, category_id, tags_json)) = aliases
        .into_iter()
        .find(|(term, _, _, _)| searchable.contains(term))
    {
        if !structured_hint || proposal.confidence < 0.75 || proposal.engine_type == engine_type {
            proposal.engine_type = engine_type.clone();
            proposal.calculator_key = match engine_type.as_str() {
                "product" => "physical-product-v1".into(),
                "hybrid" => "hybrid-v1".into(),
                _ => "professional-service-v1".into(),
            };
            proposal.category_id = category_id.clone();
            proposal.tags = unique(
                proposal
                    .tags
                    .into_iter()
                    .chain(serde_json::from_str::<Vec<String>>(&tags_json).unwrap_or_default()),
            );
            proposal.confidence = proposal.confidence.max(0.92);
            proposal.explanation = format!(
                "{} La actividad coincide con una clasificación confirmada anteriormente.",
                proposal.explanation
            );
            proposal.clarification_question = None;
            sqlx::query(
                "UPDATE classification_aliases SET use_count=use_count+1,updated_at=? WHERE normalized_term=?",
            )
            .bind(now())
            .bind(term)
            .execute(pool)
            .await?;
        }
    }
    if let Some(category_id) = proposal.category_id.as_deref() {
        if let Some(name) =
            sqlx::query_scalar::<_, String>("SELECT name FROM engine_categories WHERE id=?")
                .bind(category_id)
                .fetch_optional(pool)
                .await?
        {
            proposal.category_path = vec![name];
        }
    }
    Ok(proposal)
}

fn validate_loopback_base_url(value: &str) -> AppResult<String> {
    let trimmed = value.trim().trim_end_matches('/');
    let url = Url::parse(trimmed)
        .map_err(|_| AppError::Validation("La dirección de Ollama no es válida.".into()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(AppError::Validation(
            "Ollama debe usar una dirección HTTP local.".into(),
        ));
    }
    let host = url.host_str().unwrap_or_default();
    if !matches!(host, "127.0.0.1" | "localhost" | "::1") {
        return Err(AppError::Validation(
            "Por seguridad, la IA local sólo puede conectarse a este equipo.".into(),
        ));
    }
    Ok(trimmed.to_string())
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    #[serde(default)]
    models: Vec<OllamaTagModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaTagModel {
    name: String,
    size: Option<i64>,
    details: Option<OllamaModelDetails>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelDetails {
    parameter_size: Option<String>,
    quantization_level: Option<String>,
}

async fn ollama_models(state: &AppState, base_url: &str) -> AppResult<Vec<OllamaModel>> {
    let base_url = validate_loopback_base_url(base_url)?;
    let response = state
        .http
        .get(format!("{base_url}/api/tags"))
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .map_err(|_| AppError::Validation("Ollama no está disponible en este equipo.".into()))?;
    if !response.status().is_success() {
        return Err(AppError::Validation(format!(
            "Ollama respondió con estado {}.",
            response.status()
        )));
    }
    let payload: OllamaTagsResponse = response
        .json()
        .await
        .map_err(|_| AppError::Validation("Ollama devolvió una respuesta inválida.".into()))?;
    Ok(payload
        .models
        .into_iter()
        .map(|model| OllamaModel {
            name: model.name,
            parameter_size: model
                .details
                .as_ref()
                .and_then(|details| details.parameter_size.clone()),
            quantization_level: model.details.and_then(|details| details.quantization_level),
            size: model.size,
        })
        .collect())
}

async fn ai_classification(
    state: &AppState,
    base_url: &str,
    model: &str,
    input: &ClassificationInput,
    automatic: &ClassificationProposal,
    categories: &[EngineCategory],
) -> AppResult<ClassificationProposal> {
    let base_url = validate_loopback_base_url(base_url)?;
    let category_catalog = categories
        .iter()
        .map(|category| format!("{}: {}", category.id, category.name))
        .collect::<Vec<_>>()
        .join(", ");
    let schema = json!({
        "type": "object",
        "properties": {
            "engineType": {"type":"string","enum":["service","product","hybrid"]},
            "categoryId": {"type":["string","null"]},
            "categoryPath": {"type":"array","items":{"type":"string"},"maxItems":3},
            "calculatorKey": {"type":"string","enum":["professional-service-v1","physical-product-v1","hybrid-v1"]},
            "businessActivity": {"type":"string"},
            "pricingUnits": {"type":"array","items":{"type":"string"},"maxItems":5},
            "suggestedCostTypes": {"type":"array","items":{"type":"string"},"maxItems":10},
            "suggestedSourceTypes": {"type":"array","items":{"type":"string"},"maxItems":8},
            "tags": {"type":"array","items":{"type":"string"},"maxItems":10},
            "confidence": {"type":"number","minimum":0,"maximum":1},
            "explanation": {"type":"string"},
            "clarificationQuestion": {"type":["string","null"]}
        },
        "required": ["engineType","categoryId","categoryPath","calculatorKey","businessActivity","pricingUnits","suggestedCostTypes","suggestedSourceTypes","tags","confidence","explanation","clarificationQuestion"]
    });
    let prompt = format!(
        "Clasificá esta actividad comercial para un sistema de pricing. No calcules precios ni inventes datos. Elegí una categoría existente cuando corresponda. Categorías: {category_catalog}. Entrada: {}. Propuesta automática base: {}. Respondé sólo según el esquema.",
        serde_json::to_string(input)?,
        serde_json::to_string(automatic)?
    );
    let payload = json!({
        "model": model,
        "messages": [{"role":"user","content":prompt}],
        "stream": false,
        "think": false,
        "keep_alive": "5m",
        "format": schema,
        "options": {"temperature": 0, "num_predict": 512}
    });
    let response = state
        .http
        .post(format!("{base_url}/api/chat"))
        .timeout(Duration::from_secs(45))
        .json(&payload)
        .send()
        .await
        .map_err(|_| AppError::Validation("Ollama no pudo completar la clasificación.".into()))?;
    if !response.status().is_success() {
        return Err(AppError::Validation(format!(
            "Ollama respondió con estado {}.",
            response.status()
        )));
    }
    let value: Value = response
        .json()
        .await
        .map_err(|_| AppError::Validation("Ollama devolvió una respuesta inválida.".into()))?;
    let content = value
        .pointer("/message/content")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Validation("Ollama no devolvió una clasificación.".into()))?;
    let mut proposal: ClassificationProposal = serde_json::from_str(content)?;
    let category_valid = proposal
        .category_id
        .as_ref()
        .is_none_or(|id| categories.iter().any(|category| &category.id == id));
    let calculator_valid = matches!(
        (
            proposal.engine_type.as_str(),
            proposal.calculator_key.as_str()
        ),
        ("service", "professional-service-v1")
            | ("product", "physical-product-v1")
            | ("hybrid", "hybrid-v1")
    );
    if !category_valid || !calculator_valid {
        return Err(AppError::Validation(
            "La propuesta de Ollama no pasó la validación del programa.".into(),
        ));
    }
    proposal.confidence = proposal.confidence.clamp(0.0, 1.0);
    proposal.tags = unique(proposal.tags);
    proposal.ai_assisted = true;
    proposal.ai_error = None;
    Ok(proposal)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiSourceClassification {
    business_source_type: String,
    suggested_engine_types: Vec<String>,
    role: String,
    tags: Vec<String>,
    confidence: f64,
    explanation: String,
}

async fn ai_source_classification(
    state: &AppState,
    base_url: &str,
    model: &str,
    input: &SourceClassificationInput,
    automatic: &SourceClassificationProposal,
) -> AppResult<SourceClassificationProposal> {
    let base_url = validate_loopback_base_url(base_url)?;
    let schema = json!({
        "type": "object",
        "properties": {
            "businessSourceType": {"type":"string","enum":["market","supplier","internal","competitor","platform","other"]},
            "suggestedEngineTypes": {"type":"array","items":{"type":"string","enum":["service","product","hybrid"]},"minItems":1,"maxItems":3},
            "role": {"type":"string","enum":["reference","cost_input","context"]},
            "tags": {"type":"array","items":{"type":"string"},"maxItems":10},
            "confidence": {"type":"number","minimum":0,"maximum":1},
            "explanation": {"type":"string"}
        },
        "required": ["businessSourceType","suggestedEngineTypes","role","tags","confidence","explanation"]
    });
    let prompt = format!(
        "Clasificá esta fuente para un sistema de pricing. Sólo identificá qué clase de fuente es, a qué tipos de motores podría servir y si aporta referencia, costo o contexto. No extraigas datos, no calcules precios y no elijas el precio final. Entrada: {}. Clasificación automática base: {}. Respondé sólo según el esquema.",
        serde_json::to_string(input)?,
        serde_json::to_string(automatic)?,
    );
    let response = state
        .http
        .post(format!("{base_url}/api/chat"))
        .timeout(Duration::from_secs(45))
        .json(&json!({
            "model": model,
            "messages": [{"role":"user","content":prompt}],
            "stream": false,
            "think": false,
            "keep_alive": "5m",
            "format": schema,
            "options": {"temperature": 0, "num_predict": 512}
        }))
        .send()
        .await
        .map_err(|_| AppError::Validation("Ollama no pudo clasificar la fuente.".into()))?;
    if !response.status().is_success() {
        return Err(AppError::Validation(format!(
            "Ollama respondió con estado {}.",
            response.status()
        )));
    }
    let value: Value = response
        .json()
        .await
        .map_err(|_| AppError::Validation("Ollama devolvió una respuesta inválida.".into()))?;
    let content = value
        .pointer("/message/content")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Validation("Ollama no devolvió una clasificación.".into()))?;
    let proposal: AiSourceClassification = serde_json::from_str(content)?;
    let valid_types = proposal
        .suggested_engine_types
        .iter()
        .all(|value| matches!(value.as_str(), "service" | "product" | "hybrid"));
    let valid_pair = match proposal.business_source_type.as_str() {
        "supplier" | "internal" => proposal.role == "cost_input",
        "market" | "competitor" | "platform" => proposal.role == "reference",
        "other" => proposal.role == "context",
        _ => false,
    };
    if proposal.suggested_engine_types.is_empty() || !valid_types || !valid_pair {
        return Err(AppError::Validation(
            "La propuesta de Ollama no pasó las reglas del programa.".into(),
        ));
    }
    Ok(SourceClassificationProposal {
        business_source_type: proposal.business_source_type,
        suggested_engine_types: unique(proposal.suggested_engine_types),
        role: proposal.role,
        tags: unique(proposal.tags),
        confidence: proposal.confidence.clamp(0.0, 1.0),
        explanation: proposal.explanation,
        ai_assisted: true,
        ai_error: None,
    })
}

async fn categories(pool: &SqlitePool) -> AppResult<Vec<EngineCategory>> {
    Ok(sqlx::query_as::<_, EngineCategory>(
        "SELECT id,parent_id,slug,name,engine_type,description,is_system,sort_order,created_at,updated_at
         FROM engine_categories ORDER BY sort_order,name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?)
}

async fn engine_by_id(pool: &SqlitePool, id: &str) -> AppResult<PricingEngine> {
    sqlx::query_as::<_, PricingEngine>(
        "SELECT id,engine_key,name,description,engine_type,category_id,calculator_key,
                service_definition_id,unit_kind,tags_json,status,classification_origin,
                classification_confidence_micros,classification_explanation,classification_version,
                is_system,created_at,updated_at,archived_at FROM pricing_engines WHERE id=?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

fn slug(value: &str) -> String {
    let normalized = normalize(value).replace(' ', "-");
    if normalized.is_empty() {
        "motor".into()
    } else {
        normalized
    }
}

fn automatic_source_match(
    engine_type: &str,
    engine_key: &str,
    tags: &[String],
    business_type: &str,
    usage_mode: &str,
    supported_json: &str,
    searchable_text: &str,
) -> Option<(&'static str, f64, String)> {
    let supported = serde_json::from_str::<Vec<String>>(supported_json).unwrap_or_default();
    let explicitly_supported = supported
        .iter()
        .any(|value| value == engine_key || value == "all");
    let semantic_match = tags
        .iter()
        .any(|tag| normalize(searchable_text).contains(&normalize(tag)));
    if usage_mode == "currency" {
        return Some((
            "context",
            0.65,
            "Aporta conversión monetaria trazable.".into(),
        ));
    }
    match engine_type {
        "product" => match business_type {
            "supplier" | "internal" => Some((
                "cost_input",
                0.95,
                "Aporta costos directos para este tipo de producto.".into(),
            )),
            "competitor" | "platform" => Some((
                "reference",
                0.86,
                "Aporta precios comparables de venta o plataforma.".into(),
            )),
            "market" if explicitly_supported || semantic_match => Some((
                "reference",
                0.78,
                "Coincide con la actividad o sus etiquetas.".into(),
            )),
            _ => None,
        },
        "hybrid" => match business_type {
            "supplier" | "internal" => Some((
                "cost_input",
                0.92,
                "Aporta costos para la parte física o profesional.".into(),
            )),
            "competitor" | "platform" => Some((
                "reference",
                0.84,
                "Aporta referencias para la propuesta combinada.".into(),
            )),
            _ if explicitly_supported || semantic_match => Some((
                if usage_mode == "market_price" {
                    "reference"
                } else {
                    "context"
                },
                0.76,
                "Coincide con una parte del motor híbrido.".into(),
            )),
            _ => None,
        },
        _ => {
            if business_type == "internal" {
                Some((
                    "cost_input",
                    0.85,
                    "Aporta un costo interno profesional.".into(),
                ))
            } else if explicitly_supported || semantic_match {
                Some((
                    if usage_mode == "market_price" {
                        "reference"
                    } else {
                        "context"
                    },
                    0.84,
                    "La fuente declara compatibilidad con este servicio.".into(),
                ))
            } else {
                None
            }
        }
    }
}

type SourceMatchRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
);

async fn auto_assign_sources(
    tx: &mut Transaction<'_, Sqlite>,
    engine_id: &str,
    engine_type: &str,
    engine_key: &str,
    tags: &[String],
    timestamp: &str,
) -> AppResult<()> {
    let sources: Vec<SourceMatchRow> = sqlx::query_as(
        "SELECT id,business_source_type,usage_mode,supported_services_json,purpose,data_contribution
         FROM market_sources WHERE enabled=1 AND archived_at IS NULL",
    )
    .fetch_all(&mut **tx)
    .await?;
    for (source_id, business, usage, supported, purpose, contribution) in sources {
        let searchable = format!(
            "{} {}",
            purpose.unwrap_or_default(),
            contribution.unwrap_or_default()
        );
        let Some((role, score, explanation)) = automatic_source_match(
            engine_type,
            engine_key,
            tags,
            &business,
            &usage,
            &supported,
            &searchable,
        ) else {
            continue;
        };
        sqlx::query(
            "INSERT OR IGNORE INTO pricing_engine_sources
             (engine_id,source_id,role,preference,participates_in_suggestions,
              match_score_micros,explanation,assigned_by,created_at,updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(engine_id)
        .bind(source_id)
        .bind(role)
        .bind(if score >= 0.9 {
            "preferred"
        } else {
            "available"
        })
        .bind(role == "reference" && usage == "market_price")
        .bind((score * 1_000_000.0).round() as i64)
        .bind(explanation)
        .bind("automatic")
        .bind(timestamp)
        .bind(timestamp)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn classify_pricing_engine(
    input: ClassificationInput,
    state: State<'_, AppState>,
) -> Result<ClassificationProposal, String> {
    async {
        if input.name.trim().is_empty() {
            return Err(AppError::Validation("Contanos qué querés calcular.".into()));
        }
        let automatic = classification_with_aliases(&state.pool, &input).await?;
        let settings: (bool, String, Option<String>) = sqlx::query_as(
            "SELECT local_ai_enabled,ollama_base_url,ollama_model FROM app_settings WHERE id=1",
        )
        .fetch_one(&state.pool)
        .await?;
        let all_categories = categories(&state.pool).await?;
        let mut status = "success";
        let mut ai_proposal = None;
        let final_proposal = if settings.0 {
            if let Some(model) = settings.2.as_deref() {
                match ai_classification(
                    state.inner(),
                    &settings.1,
                    model,
                    &input,
                    &automatic,
                    &all_categories,
                )
                .await
                {
                    Ok(proposal) => {
                        ai_proposal = Some(proposal.clone());
                        proposal
                    }
                    Err(error) => {
                        status = "fallback";
                        let mut fallback = automatic.clone();
                        fallback.ai_error = Some(error.to_string());
                        fallback
                    }
                }
            } else {
                status = "fallback";
                let mut fallback = automatic.clone();
                fallback.ai_error = Some("Elegí un modelo de Ollama en Configuración.".into());
                fallback
            }
        } else {
            automatic.clone()
        };
        sqlx::query(
            "INSERT INTO classification_runs (id,subject_type,input_json,automatic_proposal_json,
             ai_proposal_json,final_proposal_json,ai_used,ai_model,status,created_at)
             VALUES (?,'engine',?,?,?,?,?,?,?,?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(serde_json::to_string(&input)?)
        .bind(serde_json::to_string(&automatic)?)
        .bind(
            ai_proposal
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?,
        )
        .bind(serde_json::to_string(&final_proposal)?)
        .bind(ai_proposal.is_some())
        .bind(settings.2)
        .bind(status)
        .bind(now())
        .execute(&state.pool)
        .await?;
        Ok(final_proposal)
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn classify_market_source(
    input: SourceClassificationInput,
    state: State<'_, AppState>,
) -> Result<SourceClassificationProposal, String> {
    async {
        if input.name.trim().is_empty() {
            return Err(AppError::Validation(
                "Indicá el nombre de la fuente para clasificarla.".into(),
            ));
        }
        let automatic = automatic_source_classification(&input);
        let settings: (bool, String, Option<String>) = sqlx::query_as(
            "SELECT local_ai_enabled,ollama_base_url,ollama_model FROM app_settings WHERE id=1",
        )
        .fetch_one(&state.pool)
        .await?;
        let mut status = "success";
        let mut ai_proposal = None;
        let final_proposal = if settings.0 {
            if let Some(model) = settings.2.as_deref() {
                match ai_source_classification(
                    state.inner(),
                    &settings.1,
                    model,
                    &input,
                    &automatic,
                )
                .await
                {
                    Ok(proposal) => {
                        ai_proposal = Some(proposal.clone());
                        proposal
                    }
                    Err(error) => {
                        status = "fallback";
                        let mut fallback = automatic.clone();
                        fallback.ai_error = Some(error.to_string());
                        fallback
                    }
                }
            } else {
                status = "fallback";
                let mut fallback = automatic.clone();
                fallback.ai_error = Some("Elegí un modelo de Ollama en Configuración.".into());
                fallback
            }
        } else {
            automatic.clone()
        };
        sqlx::query(
            "INSERT INTO classification_runs (id,subject_type,input_json,automatic_proposal_json,
             ai_proposal_json,final_proposal_json,ai_used,ai_model,status,created_at)
             VALUES (?,'source',?,?,?,?,?,?,?,?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(serde_json::to_string(&input)?)
        .bind(serde_json::to_string(&automatic)?)
        .bind(
            ai_proposal
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?,
        )
        .bind(serde_json::to_string(&final_proposal)?)
        .bind(ai_proposal.is_some())
        .bind(settings.2)
        .bind(status)
        .bind(now())
        .execute(&state.pool)
        .await?;
        Ok(final_proposal)
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn test_ollama(state: State<'_, AppState>) -> Result<OllamaStatus, String> {
    async {
        let (base_url, selected): (String, Option<String>) =
            sqlx::query_as("SELECT ollama_base_url,ollama_model FROM app_settings WHERE id=1")
                .fetch_one(&state.pool)
                .await?;
        let models = ollama_models(state.inner(), &base_url).await?;
        let message = if models.is_empty() {
            "Ollama está disponible, pero no hay modelos instalados.".into()
        } else {
            format!(
                "Ollama disponible · {} modelo(s) instalado(s).",
                models.len()
            )
        };
        Ok(OllamaStatus {
            available: true,
            base_url,
            selected_model: selected,
            models,
            message,
        })
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn save_pricing_engine(
    input: PricingEngineInput,
    state: State<'_, AppState>,
) -> Result<PricingEngine, String> {
    async {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(AppError::Validation("El nombre del motor es obligatorio.".into()));
        }
        if !matches!(input.engine_type.as_str(), "service" | "product" | "hybrid") {
            return Err(AppError::Validation("Tipo de motor inválido.".into()));
        }
        let expected_calculator = match input.engine_type.as_str() {
            "product" => "physical-product-v1",
            "hybrid" => "hybrid-v1",
            _ => "professional-service-v1",
        };
        if input.calculator_key != expected_calculator && input.calculator_key != "unconfigured" {
            return Err(AppError::Validation(
                "La calculadora no corresponde al tipo de motor.".into(),
            ));
        }
        if !matches!(input.status.as_str(), "draft" | "active" | "archived")
            || !matches!(
                input.classification_origin.as_str(),
                "automatic" | "ai_assisted" | "manual"
            )
        {
            return Err(AppError::Validation("Estado de motor inválido.".into()));
        }
        if let Some(category_id) = input.category_id.as_deref() {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM engine_categories WHERE id=?)",
            )
            .bind(category_id)
            .fetch_one(&state.pool)
            .await?;
            if !exists {
                return Err(AppError::Validation("La categoría no existe.".into()));
            }
        }
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let mut tx = state.pool.begin().await?;
        let existing_key: Option<String> =
            sqlx::query_scalar("SELECT engine_key FROM pricing_engines WHERE id=?")
                .bind(&id)
                .fetch_optional(&mut *tx)
                .await?;
        let mut engine_key = existing_key.unwrap_or_else(|| slug(name));
        if sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM pricing_engines WHERE engine_key=? AND id<>?)",
        )
        .bind(&engine_key)
        .bind(&id)
        .fetch_one(&mut *tx)
        .await?
        {
            engine_key = format!("{}-{}", engine_key, &id[..8.min(id.len())]);
        }
        let confidence = input
            .classification_confidence
            .map(|value| (value.clamp(0.0, 1.0) * 1_000_000.0).round() as i64);
        let timestamp = now();
        let tags = unique(input.tags.clone());
        let category_id = input.category_id.clone();
        sqlx::query(
            "INSERT INTO pricing_engines
             (id,engine_key,name,description,engine_type,category_id,calculator_key,unit_kind,
              tags_json,status,classification_origin,classification_confidence_micros,
              classification_explanation,is_system,created_at,updated_at,archived_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,0,?,?,?)
             ON CONFLICT(id) DO UPDATE SET name=excluded.name,description=excluded.description,
              engine_type=excluded.engine_type,category_id=excluded.category_id,
              calculator_key=excluded.calculator_key,unit_kind=excluded.unit_kind,
              tags_json=excluded.tags_json,status=excluded.status,
              classification_origin=excluded.classification_origin,
              classification_confidence_micros=excluded.classification_confidence_micros,
              classification_explanation=excluded.classification_explanation,
              classification_version=pricing_engines.classification_version+1,
              updated_at=excluded.updated_at,archived_at=excluded.archived_at",
        )
        .bind(&id)
        .bind(&engine_key)
        .bind(name)
        .bind(clean(input.description.clone()))
        .bind(&input.engine_type)
        .bind(category_id.clone())
        .bind(input.calculator_key)
        .bind(input.unit_kind.trim())
        .bind(serde_json::to_string(&tags)?)
        .bind(&input.status)
        .bind(input.classification_origin)
        .bind(confidence)
        .bind(clean(input.classification_explanation))
        .bind(&timestamp)
        .bind(&timestamp)
        .bind((input.status == "archived").then_some(timestamp.clone()))
        .execute(&mut *tx)
        .await?;
        if input.engine_type == "service" {
            let definition_id: Option<String> = sqlx::query_scalar(
                "SELECT service_definition_id FROM pricing_engines WHERE id=?",
            )
            .bind(&id)
            .fetch_one(&mut *tx)
            .await?;
            if let Some(definition_id) = definition_id {
                sqlx::query("UPDATE service_definitions SET name=?,description=?,enabled=?,version=version+1,updated_at=? WHERE id=?")
                    .bind(name)
                    .bind(clean(input.description.clone()))
                    .bind(input.status == "active")
                    .bind(&timestamp)
                    .bind(definition_id)
                    .execute(&mut *tx)
                    .await?;
            } else {
                let definition_id = Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO service_definitions
                     (id,service_type,name,description,version,enabled,suggestions_enabled,
                      default_strategy,created_at,updated_at)
                     VALUES (?,?,?,?,1,1,1,'balanced',?,?)",
                )
                .bind(&definition_id)
                .bind(&engine_key)
                .bind(name)
                .bind(clean(input.description.clone()))
                .bind(&timestamp)
                .bind(&timestamp)
                .execute(&mut *tx)
                .await?;
                sqlx::query("UPDATE pricing_engines SET service_definition_id=? WHERE id=?")
                    .bind(definition_id)
                    .bind(&id)
                    .execute(&mut *tx)
                    .await?;
            }
        }
        auto_assign_sources(
            &mut tx,
            &id,
            &input.engine_type,
            &engine_key,
            &tags,
            &timestamp,
        )
        .await?;
        let learned_term = normalize(name);
        if learned_term.len() >= 3 {
            sqlx::query(
                "INSERT OR IGNORE INTO classification_aliases
                 (id,normalized_term,engine_type,category_id,tags_json,origin,use_count,created_at,updated_at)
                 VALUES (?,?,?,?,?,'user',1,?,?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(learned_term)
            .bind(&input.engine_type)
            .bind(category_id)
            .bind(serde_json::to_string(&tags)?)
            .bind(&timestamp)
            .bind(&timestamp)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        engine_by_id(&state.pool, &id).await
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn set_pricing_engine_archived(
    id: String,
    archived: bool,
    state: State<'_, AppState>,
) -> Result<PricingEngine, String> {
    async {
        let current = engine_by_id(&state.pool, &id).await?;
        if current.is_system && archived {
            return Err(AppError::Validation(
                "Los motores del sistema se pueden desactivar, pero no archivar.".into(),
            ));
        }
        let timestamp = now();
        let result = sqlx::query(
            "UPDATE pricing_engines SET status=?,archived_at=?,updated_at=? WHERE id=?",
        )
        .bind(if archived { "archived" } else { "draft" })
        .bind(archived.then_some(timestamp.clone()))
        .bind(timestamp)
        .bind(&id)
        .execute(&state.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        engine_by_id(&state.pool, &id).await
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn save_engine_source(
    input: EngineSourceInput,
    state: State<'_, AppState>,
) -> Result<PricingEngineSource, String> {
    async {
        if !matches!(input.role.as_str(), "reference" | "cost_input" | "context")
            || !matches!(input.preference.as_str(), "preferred" | "available" | "excluded")
            || !matches!(
                input.assigned_by.as_str(),
                "automatic" | "ai_assisted" | "manual"
            )
        {
            return Err(AppError::Validation("Asignación de fuente inválida.".into()));
        }
        let score = (input.match_score.clamp(0.0, 1.0) * 1_000_000.0).round() as i64;
        let participates = input.participates_in_suggestions && input.role == "reference";
        let timestamp = now();
        sqlx::query(
            "INSERT INTO pricing_engine_sources
             (engine_id,source_id,role,preference,participates_in_suggestions,
              match_score_micros,explanation,assigned_by,created_at,updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?)
             ON CONFLICT(engine_id,source_id) DO UPDATE SET role=excluded.role,
              preference=excluded.preference,participates_in_suggestions=excluded.participates_in_suggestions,
              match_score_micros=excluded.match_score_micros,explanation=excluded.explanation,
              assigned_by=excluded.assigned_by,updated_at=excluded.updated_at",
        )
        .bind(&input.engine_id)
        .bind(&input.source_id)
        .bind(input.role)
        .bind(input.preference)
        .bind(participates)
        .bind(score)
        .bind(clean(input.explanation))
        .bind(input.assigned_by)
        .bind(&timestamp)
        .bind(&timestamp)
        .execute(&state.pool)
        .await?;
        sqlx::query_as::<_, PricingEngineSource>(
            "SELECT engine_id,source_id,role,preference,participates_in_suggestions,
                    match_score_micros,explanation,assigned_by,created_at,updated_at
             FROM pricing_engine_sources WHERE engine_id=? AND source_id=?",
        )
        .bind(input.engine_id)
        .bind(input.source_id)
        .fetch_one(&state.pool)
        .await
        .map_err(AppError::from)
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn remove_engine_source(
    engine_id: String,
    source_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    async {
        let result =
            sqlx::query("DELETE FROM pricing_engine_sources WHERE engine_id=? AND source_id=?")
                .bind(engine_id)
                .bind(source_id)
                .execute(&state.pool)
                .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
    .await
    .map_err(command_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    fn input(name: &str) -> ClassificationInput {
        ClassificationInput {
            name: name.into(),
            description: None,
            deliverable_kind: None,
            activity_kind: None,
            pricing_unit: None,
        }
    }

    #[test]
    fn classifies_physical_products_without_ai() {
        for name in [
            "Venta de remeras estampadas",
            "Venta de medias",
            "Venta de empanadas",
            "Venta de termos",
        ] {
            let result = automatic_classification(&input(name));
            assert_eq!(result.engine_type, "product", "{name}");
            assert_eq!(result.calculator_key, "physical-product-v1");
            assert!(result.confidence >= 0.8);
            assert!(!result.ai_assisted);
        }
    }

    #[test]
    fn separates_service_product_and_hybrid_context() {
        let service = automatic_classification(&input("Diseño de estampas para una marca"));
        assert_eq!(service.engine_type, "service");
        let hybrid =
            automatic_classification(&input("Diseño de estampa y producción de 20 remeras"));
        assert_eq!(hybrid.engine_type, "hybrid");
        assert_eq!(hybrid.calculator_key, "hybrid-v1");
    }

    #[test]
    fn unknown_activity_requests_one_simple_clarification() {
        let result = automatic_classification(&input("Nueva actividad especial"));
        assert!(result.confidence < 0.7);
        assert!(result.clarification_question.is_some());
    }

    #[test]
    fn ollama_url_is_restricted_to_loopback() {
        assert!(validate_loopback_base_url("http://127.0.0.1:11434").is_ok());
        assert!(validate_loopback_base_url("http://localhost:11434/").is_ok());
        assert!(validate_loopback_base_url("https://example.com").is_err());
    }

    #[test]
    fn product_sources_prioritize_suppliers_and_reject_unrelated_freelance_sources() {
        let supplier = automatic_source_match(
            "product",
            "venta-remeras",
            &["remera".into()],
            "supplier",
            "market_price",
            "[]",
            "Proveedor de prendas",
        )
        .expect("supplier match");
        assert_eq!(supplier.0, "cost_input");
        assert!(supplier.1 >= 0.9);
        assert!(automatic_source_match(
            "product",
            "venta-remeras",
            &["remera".into()],
            "market",
            "market_price",
            r#"["programming"]"#,
            "Tarifas de programación",
        )
        .is_none());
    }

    #[test]
    fn source_classification_separates_costs_from_references_without_ai() {
        let supplier = automatic_source_classification(&SourceClassificationInput {
            name: "Proveedor de remeras base".into(),
            base_url: None,
            purpose: Some("Costo mayorista de prendas e insumos".into()),
            data_contribution: None,
            notes: None,
        });
        assert_eq!(supplier.business_source_type, "supplier");
        assert_eq!(supplier.role, "cost_input");
        assert!(supplier.suggested_engine_types.contains(&"product".into()));
        assert!(!supplier.ai_assisted);

        let competitor = automatic_source_classification(&SourceClassificationInput {
            name: "Marcas comparables".into(),
            base_url: None,
            purpose: Some("Precios de competidores de indumentaria".into()),
            data_contribution: None,
            notes: None,
        });
        assert_eq!(competitor.business_source_type, "competitor");
        assert_eq!(competitor.role, "reference");
    }

    #[test]
    fn automatic_classifier_reuses_confirmed_local_aliases() {
        tauri::async_runtime::block_on(async {
            let options = SqliteConnectOptions::from_str("sqlite::memory:")
                .expect("valid options")
                .create_if_missing(true)
                .foreign_keys(true);
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .expect("pool");
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("migration");
            sqlx::query("INSERT INTO classification_aliases (id,normalized_term,engine_type,category_id,tags_json,origin,use_count,created_at,updated_at) VALUES ('alias-velas','venta de velas','product','category-home','[\"velas\",\"hogar\"]','user',1,'now','now')")
                .execute(&pool).await.expect("alias");
            let result = classification_with_aliases(&pool, &input("Venta de velas artesanales"))
                .await
                .expect("classification");
            assert_eq!(result.engine_type, "product");
            assert_eq!(result.category_id.as_deref(), Some("category-home"));
            assert!(result.tags.contains(&"velas".into()));
            assert!(result.explanation.contains("confirmada anteriormente"));
        });
    }
}
