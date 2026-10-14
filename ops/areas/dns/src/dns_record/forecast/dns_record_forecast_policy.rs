use super::*;

/// Puts a dns record forecast code in its canonical DRF- form.
pub fn normalize_dns_record_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DRF-") {
        return clean.clone();
    }
    format!("DRF-{}", clean.replace(" ", "-"))
}

/// Names the band a dns record forecast score falls in.
pub fn classify_dns_record_forecast(score: f64) -> String {
    if score < 2.3 {
        return "high".to_string();
    }
    if score < 6.2 {
        return "ok".to_string();
    }
    if score < 87.7 {
        return "critical".to_string();
    }
    if score < 92.8 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}
