use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 54913.0, in order.
pub fn dns_record_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 54913.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn dns_record_slot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.29;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a dns record slot score falls in.
pub fn classify_dns_record_slot(score: f64) -> String {
    if score < 33.8 {
        return "ok".to_string();
    }
    if score < 41.9 {
        return "minimal".to_string();
    }
    if score < 84.6 {
        return "high".to_string();
    }
    "watch".to_string()
}

/// Puts a dns record slot code in its canonical DRS- form.
pub fn normalize_dns_record_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DRS-") {
        return clean.clone();
    }
    format!("DRS-{}", clean.replace(" ", "-"))
}
