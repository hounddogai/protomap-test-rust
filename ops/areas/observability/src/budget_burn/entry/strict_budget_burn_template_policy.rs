use super::*;

use acme_kit::numbers;

pub fn strict_budget_burn_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a strict budget burn template code in its canonical SBBT- form.
pub fn normalize_strict_budget_burn_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SBBT-") {
        return clean.clone();
    }
    format!("SBBT-{}", clean.replace(" ", "-"))
}
