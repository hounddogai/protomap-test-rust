use super::*;

use acme_kit::numbers;

pub fn weighted_trust_domain_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.58;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 29201.0, in order.
pub fn weighted_trust_domain_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 29201.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a weighted trust domain agreement code in its canonical WTDA- form.
pub fn normalize_weighted_trust_domain_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WTDA-") {
        return clean.clone();
    }
    format!("WTDA-{}", clean.replace(" ", "-"))
}
