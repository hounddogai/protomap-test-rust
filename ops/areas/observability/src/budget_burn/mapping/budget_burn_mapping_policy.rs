use super::*;

/// Names the band a budget burn mapping score falls in.
pub fn classify_budget_burn_mapping(score: f64) -> String {
    if score < 13.3 {
        return "medium".to_string();
    }
    if score < 40.0 {
        return "minimal".to_string();
    }
    "high".to_string()
}

/// Puts a budget burn mapping code in its canonical BBM- form.
pub fn normalize_budget_burn_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BBM-") {
        return clean.clone();
    }
    format!("BBM-{}", clean.replace(" ", "-"))
}
