use crate::error::{command_error, AppError, AppResult};

const MAX_PDF_BYTES: usize = 5 * 1024 * 1024;
const MAX_EXTRACTED_CHARS: usize = 250_000;

fn base64_value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn decode_pdf_data_url(data_url: &str) -> AppResult<Vec<u8>> {
    let (metadata, encoded) = data_url
        .split_once(',')
        .ok_or_else(|| AppError::Validation("El archivo PDF no tiene un formato válido.".into()))?;
    if !metadata.starts_with("data:") || !metadata.contains(";base64") {
        return Err(AppError::Validation(
            "Seleccioná un PDF válido para importar.".into(),
        ));
    }
    if encoded.len() > MAX_PDF_BYTES.saturating_mul(2) {
        return Err(AppError::Validation(
            "El PDF supera el límite de 5 MB para importar.".into(),
        ));
    }
    let mut output = Vec::with_capacity(encoded.len() * 3 / 4);
    let mut chunk = [0_u8; 4];
    let mut count = 0;
    for byte in encoded.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
        chunk[count] = if byte == b'=' {
            64
        } else {
            base64_value(byte)
                .ok_or_else(|| AppError::Validation("El PDF no contiene Base64 válido.".into()))?
        };
        count += 1;
        if count != 4 {
            continue;
        }
        if chunk[0] == 64 || chunk[1] == 64 {
            return Err(AppError::Validation(
                "El PDF no contiene Base64 válido.".into(),
            ));
        }
        output.push((chunk[0] << 2) | (chunk[1] >> 4));
        if chunk[2] != 64 {
            output.push((chunk[1] << 4) | (chunk[2] >> 2));
        }
        if chunk[3] != 64 && chunk[2] != 64 {
            output.push((chunk[2] << 6) | chunk[3]);
        }
        if output.len() > MAX_PDF_BYTES {
            return Err(AppError::Validation(
                "El PDF supera el límite de 5 MB para importar.".into(),
            ));
        }
        if chunk[2] == 64 || chunk[3] == 64 {
            if byte != b'=' {
                return Err(AppError::Validation(
                    "El PDF no contiene Base64 válido.".into(),
                ));
            }
        }
        count = 0;
    }
    if count != 0 || output.is_empty() || !output.starts_with(b"%PDF-") {
        return Err(AppError::Validation(
            "El archivo seleccionado no es un PDF válido.".into(),
        ));
    }
    Ok(output)
}

#[tauri::command]
pub fn extract_economy_pdf_text(data_url: String) -> Result<String, String> {
    (|| {
        let bytes = decode_pdf_data_url(&data_url)?;
        let text = pdf_extract::extract_text_from_mem(&bytes).map_err(|_| {
            AppError::Validation(
                "No se pudo extraer texto de este PDF. Usá un PDF con texto seleccionable o importá el JSON, TXT o MD generado por la IA.".into(),
            )
        })?;
        let text = text.chars().take(MAX_EXTRACTED_CHARS).collect::<String>();
        if text.trim().is_empty() {
            return Err(AppError::Validation(
                "El PDF no contiene texto importable. Si es una imagen escaneada, pedile a la IA un JSON, TXT o MD.".into(),
            ));
        }
        Ok(text)
    })()
    .map_err(command_error)
}

#[cfg(test)]
mod tests {
    use super::decode_pdf_data_url;

    fn text_pdf(text: &str) -> Vec<u8> {
        let escaped = text
            .replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)");
        let content = format!("BT\n/F1 12 Tf\n72 720 Td\n({escaped}) Tj\nET\n");
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /Resources << /Font << /F1 5 0 R >> >> /MediaBox [0 0 612 792] /Contents 4 0 R >>".to_string(),
            format!("<< /Length {} >>\nstream\n{content}endstream", content.len()),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
        ];
        let mut pdf = b"%PDF-1.4\n".to_vec();
        let mut offsets = Vec::new();
        for (index, object) in objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", index + 1, object).as_bytes());
        }
        let xref = pdf.len();
        pdf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
        for offset in offsets {
            pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF").as_bytes(),
        );
        pdf
    }

    #[test]
    fn decodes_a_small_pdf_data_url() {
        let decoded =
            decode_pdf_data_url("data:application/pdf;base64,JVBERi0=").expect("PDF data URL");
        assert_eq!(decoded, b"%PDF-");
    }

    #[test]
    fn accepts_a_pdf_with_generic_browser_metadata() {
        let decoded = decode_pdf_data_url("data:application/octet-stream;base64,JVBERi0=")
            .expect("PDF bytes determine the file type");
        assert_eq!(decoded, b"%PDF-");
    }

    #[test]
    fn rejects_non_pdf_data() {
        assert!(decode_pdf_data_url("data:text/plain;base64,aG9sYQ==").is_err());
    }

    #[test]
    fn extracts_text_from_a_text_based_pdf() {
        let text =
            pdf_extract::extract_text_from_mem(&text_pdf("moneda: ARS")).expect("extractable PDF");
        assert!(text.contains("moneda: ARS"));
    }
}
