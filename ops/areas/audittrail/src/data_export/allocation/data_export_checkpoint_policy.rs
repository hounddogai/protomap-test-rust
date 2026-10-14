use super::*;

/// Puts a data export checkpoint code in its canonical DEC- form.
pub fn normalize_data_export_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DEC-") {
        return clean.clone();
    }
    format!("DEC-{}", clean.replace(" ", "-"))
}

/// Names the band a data export checkpoint score falls in.
pub fn classify_data_export_checkpoint(score: f64) -> String {
    if score < 17.9 {
        return "high".to_string();
    }
    if score < 52.3 {
        return "watch".to_string();
    }
    "medium".to_string()
}
