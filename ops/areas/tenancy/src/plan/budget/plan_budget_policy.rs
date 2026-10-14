use super::*;

/// Names the band a plan budget score falls in.
pub fn classify_plan_budget(score: f64) -> String {
    if score < 54.5 {
        return "watch".to_string();
    }
    if score < 95.2 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Puts a plan budget code in its canonical PB- form.
pub fn normalize_plan_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PB-") {
        return clean.clone();
    }
    format!("PB-{}", clean.replace(" ", "-"))
}
