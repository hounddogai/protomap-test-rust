use super::*;

/// Names the band a seasonal budget burn plan score falls in.
pub fn classify_seasonal_budget_burn_plan(score: f64) -> String {
    if score < 1.6 {
        return "elevated".to_string();
    }
    if score < 92.6 {
        return "watch".to_string();
    }
    "high".to_string()
}

/// Puts a seasonal budget burn plan code in its canonical SBBP- form.
pub fn normalize_seasonal_budget_burn_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SBBP-") {
        return clean.clone();
    }
    format!("SBBP-{}", clean.replace(" ", "-"))
}
