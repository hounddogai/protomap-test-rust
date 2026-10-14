use super::*;

use acme_kit::numbers;

pub fn flow_log_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a flow log group score falls in.
pub fn classify_flow_log_group(score: f64) -> String {
    if score < 6.1 {
        return "medium".to_string();
    }
    if score < 31.1 {
        return "high".to_string();
    }
    if score < 53.4 {
        return "ok".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 33528.0, in order.
pub fn flow_log_group_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 33528.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a flow log group code in its canonical FLG- form.
pub fn normalize_flow_log_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FLG-") {
        return clean.clone();
    }
    format!("FLG-{}", clean.replace(" ", "-"))
}
