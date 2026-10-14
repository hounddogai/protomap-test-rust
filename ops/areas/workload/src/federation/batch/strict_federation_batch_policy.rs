use super::*;

/// Puts a strict federation batch code in its canonical SFB- form.
pub fn normalize_strict_federation_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SFB-") {
        return clean.clone();
    }
    format!("SFB-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 79483.0, in order.
pub fn strict_federation_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 79483.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
