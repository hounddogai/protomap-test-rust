use super::*;

use acme_kit::numbers;

pub fn default_flow_log_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a default flow log budget code in its canonical DFLB- form.
pub fn normalize_default_flow_log_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DFLB-") {
        return clean.clone();
    }
    format!("DFLB-{}", clean.replace(" ", "-"))
}
