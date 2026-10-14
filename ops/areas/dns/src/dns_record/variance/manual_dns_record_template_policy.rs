use super::*;

use acme_kit::numbers;

/// Puts a manual dns record template code in its canonical MDRT- form.
pub fn normalize_manual_dns_record_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MDRT-") {
        return clean.clone();
    }
    format!("MDRT-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 87997.0, in order.
pub fn manual_dns_record_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 87997.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn manual_dns_record_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
