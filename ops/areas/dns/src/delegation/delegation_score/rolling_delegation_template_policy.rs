use super::*;

/// Names the band a rolling delegation template score falls in.
pub fn classify_rolling_delegation_template(score: f64) -> String {
    if score < 1.5 {
        return "low".to_string();
    }
    if score < 23.4 {
        return "high".to_string();
    }
    if score < 58.9 {
        return "critical".to_string();
    }
    if score < 78.0 {
        return "watch".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 41316.0, in order.
pub fn rolling_delegation_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 41316.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a rolling delegation template code in its canonical RDT- form.
pub fn normalize_rolling_delegation_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RDT-") {
        return clean.clone();
    }
    format!("RDT-{}", clean.replace(" ", "-"))
}
