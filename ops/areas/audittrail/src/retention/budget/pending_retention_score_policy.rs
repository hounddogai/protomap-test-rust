use super::*;

/// Puts a pending retention score code in its canonical PRS- form.
pub fn normalize_pending_retention_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRS-") {
        return clean.clone();
    }
    format!("PRS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 58043.0, in order.
pub fn pending_retention_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 58043.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
