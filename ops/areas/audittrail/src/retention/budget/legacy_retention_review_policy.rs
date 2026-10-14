use super::*;

/// Puts a legacy retention review code in its canonical LRR- form.
pub fn normalize_legacy_retention_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LRR-") {
        return clean.clone();
    }
    format!("LRR-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 89874.0, in order.
pub fn legacy_retention_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 89874.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
