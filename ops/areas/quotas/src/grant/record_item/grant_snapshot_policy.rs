use super::*;

/// Puts a grant snapshot code in its canonical GS- form.
pub fn normalize_grant_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GS-") {
        return clean.clone();
    }
    format!("GS-{}", clean.replace(" ", "-"))
}

/// Names the band a grant snapshot score falls in.
pub fn classify_grant_snapshot(score: f64) -> String {
    if score < 27.1 {
        return "medium".to_string();
    }
    if score < 69.4 {
        return "watch".to_string();
    }
    if score < 73.7 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 25412.0, in order.
pub fn grant_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 25412.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
