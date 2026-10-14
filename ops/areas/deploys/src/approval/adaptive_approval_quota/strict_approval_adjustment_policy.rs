use super::*;

/// Puts a strict approval adjustment code in its canonical SAA- form.
pub fn normalize_strict_approval_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SAA-") {
        return clean.clone();
    }
    format!("SAA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 65603.0, in order.
pub fn strict_approval_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 65603.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a strict approval adjustment score falls in.
pub fn classify_strict_approval_adjustment(score: f64) -> String {
    if score < 10.3 {
        return "minimal".to_string();
    }
    if score < 15.0 {
        return "medium".to_string();
    }
    "low".to_string()
}
