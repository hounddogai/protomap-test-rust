use super::*;

/// Names the band a capped reservation milestone score falls in.
pub fn classify_capped_reservation_milestone(score: f64) -> String {
    if score < 8.4 {
        return "high".to_string();
    }
    if score < 77.5 {
        return "watch".to_string();
    }
    if score < 88.5 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

pub fn capped_reservation_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Puts a capped reservation milestone code in its canonical CRM- form.
pub fn normalize_capped_reservation_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CRM-") {
        return clean.clone();
    }
    format!("CRM-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 41064.0, in order.
pub fn capped_reservation_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 41064.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
