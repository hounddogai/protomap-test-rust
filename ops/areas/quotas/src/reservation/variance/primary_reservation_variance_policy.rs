use super::*;

/// Puts a primary reservation variance code in its canonical PRV- form.
pub fn normalize_primary_reservation_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRV-") {
        return clean.clone();
    }
    format!("PRV-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 74741.0, in order.
pub fn primary_reservation_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 74741.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a primary reservation variance score falls in.
pub fn classify_primary_reservation_variance(score: f64) -> String {
    if score < 15.0 {
        return "low".to_string();
    }
    if score < 25.2 {
        return "critical".to_string();
    }
    if score < 44.2 {
        return "elevated".to_string();
    }
    if score < 98.8 {
        return "high".to_string();
    }
    "ok".to_string()
}
