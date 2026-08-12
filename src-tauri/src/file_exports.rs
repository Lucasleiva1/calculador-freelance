use std::{fs, path::PathBuf};

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

struct EconomyTemplate {
    file_name: String,
    filter_name: &'static str,
    extension: &'static str,
    contents: String,
}

fn economy_template(
    kind: &str,
    engine_name: Option<&str>,
    currency: Option<&str>,
) -> Result<EconomyTemplate, String> {
    let activity = engine_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("ACTIVIDAD SIN ESPECIFICAR");
    let selected_currency = match currency {
        Some("USD") => "USD",
        _ => "ARS",
    };
    let activity_slug = if activity.to_lowercase().contains("program") {
        "programacion"
    } else if activity.to_lowercase().contains("video") {
        "edicion-video"
    } else if activity.to_lowercase().contains("estampa") {
        "diseno-estampas"
    } else {
        "actividad"
    };
    let template = match kind {
        "ai-guide" => EconomyTemplate {
            file_name: format!(
                "guia-ia-{activity_slug}-{}.md",
                selected_currency.to_lowercase()
            ),
            filter_name: "Documento Markdown",
            extension: "md",
            contents: include_str!("../../public/templates/prompt-para-chatgpt-mi-economia.md")
                .to_string(),
        },
        "json-template" => EconomyTemplate {
            file_name: format!(
                "economia-{activity_slug}-{}.json",
                selected_currency.to_lowercase()
            ),
            filter_name: "Archivo JSON",
            extension: "json",
            contents: include_str!("../../public/templates/economia-para-importar.json")
                .to_string(),
        },
        _ => return Err("El archivo solicitado no existe.".into()),
    };
    Ok(EconomyTemplate {
        contents: template
            .contents
            .replace("{{ACTIVIDAD}}", activity)
            .replace("{{MONEDA}}", selected_currency),
        ..template
    })
}

#[tauri::command]
pub async fn save_economy_template(
    kind: String,
    engine_name: Option<String>,
    currency: Option<String>,
    app: AppHandle,
) -> Result<Option<String>, String> {
    let template = economy_template(&kind, engine_name.as_deref(), currency.as_deref())?;
    tauri::async_runtime::spawn_blocking(move || {
        let selected = app
            .dialog()
            .file()
            .set_title("Guardar archivo de economía")
            .set_file_name(&template.file_name)
            .add_filter(template.filter_name, &[template.extension])
            .blocking_save_file();

        let Some(selected) = selected else {
            return Ok(None);
        };
        let mut path = selected
            .into_path()
            .map_err(|error| format!("No se pudo interpretar la ubicación elegida: {error}"))?;
        ensure_extension(&mut path, template.extension);
        fs::write(&path, &template.contents)
            .map_err(|error| format!("No se pudo guardar el archivo: {error}"))?;
        Ok(Some(path.to_string_lossy().into_owned()))
    })
    .await
    .map_err(|error| format!("No se pudo abrir el diálogo de guardado: {error}"))?
}

fn ensure_extension(path: &mut PathBuf, extension: &str) {
    if path.extension().is_none() {
        path.set_extension(extension);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{economy_template, ensure_extension};

    #[test]
    fn exposes_both_packaged_economy_files() {
        let guide = economy_template("ai-guide", Some("Programación"), Some("ARS")).expect("guide");
        let json =
            economy_template("json-template", Some("Programación"), Some("ARS")).expect("json");
        assert!(guide.contents.contains("perfil manual completo"));
        assert!(guide.contents.contains("Programación"));
        assert_eq!(guide.file_name, "guia-ia-programacion-ars.md");
        assert!(!guide.contents.contains("{{ACTIVIDAD}}"));
        assert!(!guide.contents.contains(": null"));
        assert!(json.contents.contains("\"moneda\""));
        assert!(json.contents.contains("\"actividad\": \"Programación\""));
        assert_eq!(json.file_name, "economia-programacion-ars.json");
        assert!(!json.contents.contains(": null"));
        let video_guide = economy_template("ai-guide", Some("Edición de video"), Some("USD"))
            .expect("video guide");
        let video_json = economy_template("json-template", Some("Edición de video"), Some("USD"))
            .expect("video json");
        for file in [&video_guide, &video_json] {
            assert!(file.contents.contains("Edición de video"));
            assert!(file.contents.contains("USD"));
            assert!(!file.contents.contains(": null"));
            assert!(!file.contents.contains("{{ACTIVIDAD}}"));
        }
        assert_eq!(video_guide.file_name, "guia-ia-edicion-video-usd.md");
        assert_eq!(video_json.file_name, "economia-edicion-video-usd.json");
        let print_guide = economy_template("ai-guide", Some("Diseño de estampas"), Some("ARS"))
            .expect("print design guide");
        let print_json = economy_template("json-template", Some("Diseño de estampas"), Some("ARS"))
            .expect("print design json");
        assert_eq!(print_guide.file_name, "guia-ia-diseno-estampas-ars.md");
        assert_eq!(print_json.file_name, "economia-diseno-estampas-ars.json");
        assert!(print_guide.contents.contains("Diseño de estampas"));
        assert!(!print_guide.contents.contains(": null"));
        assert!(economy_template("unknown", None, None).is_err());
    }

    #[test]
    fn adds_the_expected_extension_only_when_missing() {
        let mut without_extension = PathBuf::from("guia-economia");
        ensure_extension(&mut without_extension, "md");
        assert_eq!(without_extension, PathBuf::from("guia-economia.md"));

        let mut existing = PathBuf::from("mi-guia.txt");
        ensure_extension(&mut existing, "md");
        assert_eq!(existing, PathBuf::from("mi-guia.txt"));
    }
}
