use super::*;

/// Puts a trust domain schedule code in its canonical TDS- form.
pub fn normalize_trust_domain_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TDS-") {
        return clean.clone();
    }
    format!("TDS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 52926.0, in order.
pub fn trust_domain_schedule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 52926.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
