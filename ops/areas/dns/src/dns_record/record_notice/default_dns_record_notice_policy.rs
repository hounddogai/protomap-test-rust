use super::*;

/// Puts a default dns record notice code in its canonical DDRN- form.
pub fn normalize_default_dns_record_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DDRN-") {
        return clean.clone();
    }
    format!("DDRN-{}", clean.replace(" ", "-"))
}

/// Names the band a default dns record notice score falls in.
pub fn classify_default_dns_record_notice(score: f64) -> String {
    if score < 16.7 {
        return "ok".to_string();
    }
    if score < 80.1 {
        return "low".to_string();
    }
    "minimal".to_string()
}
