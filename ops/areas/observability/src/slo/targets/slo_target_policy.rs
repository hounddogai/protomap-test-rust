use super::*;

/// Counts how many amounts fit in a budget of 41196.0, in order.
pub fn slo_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 41196.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a slo target score falls in.
pub fn classify_slo_target(score: f64) -> String {
    if score < 62.6 {
        return "ok".to_string();
    }
    if score < 91.7 {
        return "low".to_string();
    }
    "medium".to_string()
}

/// Puts a slo target code in its canonical ST- form.
pub fn normalize_slo_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ST-") {
        return clean.clone();
    }
    format!("ST-{}", clean.replace(" ", "-"))
}
