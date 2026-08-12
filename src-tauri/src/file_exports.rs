use std::{fs, path::PathBuf};

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

struct EconomyTemplate {
    file_name: &'static str,
    filter_name: &'static str,
    extension: &'static str,
    contents: &'static str,
}

fn economy_template(kind: &str) -> Result<EconomyTemplate, String> {
    match kind {
        "ai-guide" => Ok(EconomyTemplate {
            file_name: "guia-para-chatgpt-mi-economia.md",
            filter_name: "Documento Markdown",
            extension: "md",
            contents: include_str!("../../public/templates/prompt-para-chatgpt-mi-economia.md"),
        }),
        "json-template" => Ok(EconomyTemplate {
            file_name: "economia-para-importar.json",
            filter_name: "Archivo JSON",
            extension: "json",
            contents: include_str!("../../public/templates/economia-para-importar.json"),
        }),
        _ => Err("El archivo solicitado no existe.".into()),
    }
}

#[tauri::command]
pub async fn save_economy_template(
    kind: String,
    app: AppHandle,
) -> Result<Option<String>, String> {
    let template = economy_template(&kind)?;
    tauri::async_runtime::spawn_blocking(move || {
        let selected = app
            .dialog()
            .file()
            .set_title("Guardar archivo de economía")
            .set_file_name(template.file_name)
            .add_filter(template.filter_name, &[template.extension])
            .blocking_save_file();

        let Some(selected) = selected else {
            return Ok(None);
        };
        let mut path = selected
            .into_path()
            .map_err(|error| format!("No se pudo interpretar la ubicación elegida: {error}"))?;
        ensure_extension(&mut path, template.extension);
        fs::write(&path, template.contents)
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
        let guide = economy_template("ai-guide").expect("guide");
        let json = economy_template("json-template").expect("json");
        assert!(guide.contents.contains("econom"));
        assert!(json.contents.contains("\"moneda\""));
        assert!(economy_template("unknown").is_err());
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
