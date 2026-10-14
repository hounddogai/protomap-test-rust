use super::*;

/// Counts how many amounts fit in a budget of 66043.0, in order.
pub fn weighted_timeline_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 66043.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a weighted timeline plan code in its canonical WTP- form.
pub fn normalize_weighted_timeline_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WTP-") {
        return clean.clone();
    }
    format!("WTP-{}", clean.replace(" ", "-"))
}

/// Names the band a weighted timeline plan score falls in.
pub fn classify_weighted_timeline_plan(score: f64) -> String {
    if score < 34.2 {
        return "critical".to_string();
    }
    if score < 92.6 {
        return "ok".to_string();
    }
    "minimal".to_string()
}
