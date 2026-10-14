use super::*;

/// Counts how many amounts fit in a budget of 23347.0, in order.
pub fn pending_freeze_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 23347.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a pending freeze rollup code in its canonical PFR- form.
pub fn normalize_pending_freeze_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PFR-") {
        return clean.clone();
    }
    format!("PFR-{}", clean.replace(" ", "-"))
}

/// Names the band a pending freeze rollup score falls in.
pub fn classify_pending_freeze_rollup(score: f64) -> String {
    if score < 46.0 {
        return "medium".to_string();
    }
    if score < 54.4 {
        return "elevated".to_string();
    }
    if score < 72.8 {
        return "low".to_string();
    }
    if score < 90.3 {
        return "ok".to_string();
    }
    "critical".to_string()
}
