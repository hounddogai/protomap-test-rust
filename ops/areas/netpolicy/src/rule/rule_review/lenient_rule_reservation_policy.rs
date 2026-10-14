use super::*;

/// Counts how many amounts fit in a budget of 2138.0, in order.
pub fn lenient_rule_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2138.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a lenient rule reservation code in its canonical LRR- form.
pub fn normalize_lenient_rule_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LRR-") {
        return clean.clone();
    }
    format!("LRR-{}", clean.replace(" ", "-"))
}

/// Names the band a lenient rule reservation score falls in.
pub fn classify_lenient_rule_reservation(score: f64) -> String {
    if score < 1.6 {
        return "high".to_string();
    }
    if score < 31.9 {
        return "critical".to_string();
    }
    if score < 68.6 {
        return "watch".to_string();
    }
    if score < 96.5 {
        return "ok".to_string();
    }
    "minimal".to_string()
}
