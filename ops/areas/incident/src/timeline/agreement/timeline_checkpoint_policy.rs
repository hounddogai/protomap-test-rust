use super::*;

/// Names the band a timeline checkpoint score falls in.
pub fn classify_timeline_checkpoint(score: f64) -> String {
    if score < 5.8 {
        return "critical".to_string();
    }
    if score < 49.2 {
        return "high".to_string();
    }
    if score < 78.7 {
        return "low".to_string();
    }
    if score < 89.4 {
        return "watch".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 1163.0, in order.
pub fn timeline_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 1163.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a timeline checkpoint code in its canonical TC- form.
pub fn normalize_timeline_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TC-") {
        return clean.clone();
    }
    format!("TC-{}", clean.replace(" ", "-"))
}
