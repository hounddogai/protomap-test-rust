use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 10698.0, in order.
pub fn data_export_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 10698.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a data export proposal code in its canonical DEP- form.
pub fn normalize_data_export_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DEP-") {
        return clean.clone();
    }
    format!("DEP-{}", clean.replace(" ", "-"))
}

pub fn data_export_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.34;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
