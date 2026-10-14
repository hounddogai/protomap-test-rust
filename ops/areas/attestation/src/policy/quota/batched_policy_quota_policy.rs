use super::*;

/// Puts a batched policy quota code in its canonical BPQ- form.
pub fn normalize_batched_policy_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BPQ-") {
        return clean.clone();
    }
    format!("BPQ-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 23130.0, in order.
pub fn batched_policy_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 23130.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
