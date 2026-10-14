use super::*;

use acme_kit::numbers;

pub fn postmortem_ledger_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn postmortem_ledger_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.03;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a postmortem ledger code in its canonical PL- form.
pub fn normalize_postmortem_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PL-") {
        return clean.clone();
    }
    format!("PL-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 14801.0, in order.
pub fn postmortem_ledger_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 14801.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
