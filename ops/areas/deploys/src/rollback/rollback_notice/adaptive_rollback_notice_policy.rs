use super::*;

/// Counts how many amounts fit in a budget of 4853.0, in order.
pub fn adaptive_rollback_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 4853.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn adaptive_rollback_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Names the band a adaptive rollback notice score falls in.
pub fn classify_adaptive_rollback_notice(score: f64) -> String {
    if score < 35.5 {
        return "watch".to_string();
    }
    if score < 55.0 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

/// Puts a adaptive rollback notice code in its canonical ARN- form.
pub fn normalize_adaptive_rollback_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ARN-") {
        return clean.clone();
    }
    format!("ARN-{}", clean.replace(" ", "-"))
}
