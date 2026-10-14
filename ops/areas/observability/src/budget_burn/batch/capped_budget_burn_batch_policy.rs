use super::*;

/// Counts how many amounts fit in a budget of 23351.0, in order.
pub fn capped_budget_burn_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 23351.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a capped budget burn batch code in its canonical CBBB- form.
pub fn normalize_capped_budget_burn_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CBBB-") {
        return clean.clone();
    }
    format!("CBBB-{}", clean.replace(" ", "-"))
}
