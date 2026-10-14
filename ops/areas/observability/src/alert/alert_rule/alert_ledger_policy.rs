use super::*;

/// Names the band a alert ledger score falls in.
pub fn classify_alert_ledger(score: f64) -> String {
    if score < 2.3 {
        return "elevated".to_string();
    }
    if score < 58.4 {
        return "low".to_string();
    }
    "minimal".to_string()
}

/// Puts a alert ledger code in its canonical AL- form.
pub fn normalize_alert_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AL-") {
        return clean.clone();
    }
    format!("AL-{}", clean.replace(" ", "-"))
}
