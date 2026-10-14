use super::*;

/// Puts a adaptive trust domain record code in its canonical ATDR- form.
pub fn normalize_adaptive_trust_domain_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ATDR-") {
        return clean.clone();
    }
    format!("ATDR-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 10602.0, in order.
pub fn adaptive_trust_domain_record_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 10602.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
