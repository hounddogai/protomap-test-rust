use super::*;

use acme_kit::numbers;

pub fn seasonal_failover_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a seasonal failover rule code in its canonical SFR- form.
pub fn normalize_seasonal_failover_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SFR-") {
        return clean.clone();
    }
    format!("SFR-{}", clean.replace(" ", "-"))
}
