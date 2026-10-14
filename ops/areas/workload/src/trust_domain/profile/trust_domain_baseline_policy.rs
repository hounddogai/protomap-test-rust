use super::*;

/// Puts a trust domain baseline code in its canonical TDB- form.
pub fn normalize_trust_domain_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TDB-") {
        return clean.clone();
    }
    format!("TDB-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 49842.0, in order.
pub fn trust_domain_baseline_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 49842.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
