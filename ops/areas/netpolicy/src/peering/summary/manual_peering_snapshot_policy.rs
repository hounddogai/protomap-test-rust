use super::*;

use acme_kit::numbers;

/// Puts a manual peering snapshot code in its canonical MPS- form.
pub fn normalize_manual_peering_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MPS-") {
        return clean.clone();
    }
    format!("MPS-{}", clean.replace(" ", "-"))
}

pub fn manual_peering_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 15122.0, in order.
pub fn manual_peering_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 15122.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a manual peering snapshot score falls in.
pub fn classify_manual_peering_snapshot(score: f64) -> String {
    if score < 47.0 {
        return "ok".to_string();
    }
    if score < 60.4 {
        return "watch".to_string();
    }
    "medium".to_string()
}
