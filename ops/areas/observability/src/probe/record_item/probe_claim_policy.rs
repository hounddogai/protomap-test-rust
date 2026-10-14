use super::*;

use acme_kit::numbers;

/// Names the band a probe claim score falls in.
pub fn classify_probe_claim(score: f64) -> String {
    if score < 1.2 {
        return "low".to_string();
    }
    if score < 22.2 {
        return "elevated".to_string();
    }
    if score < 45.3 {
        return "ok".to_string();
    }
    if score < 72.6 {
        return "watch".to_string();
    }
    "high".to_string()
}

/// Puts a probe claim code in its canonical PC- form.
pub fn normalize_probe_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PC-") {
        return clean.clone();
    }
    format!("PC-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 11933.0, in order.
pub fn probe_claim_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 11933.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn probe_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
