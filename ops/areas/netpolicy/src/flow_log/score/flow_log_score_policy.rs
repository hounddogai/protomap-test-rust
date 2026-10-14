use super::*;

/// Counts how many amounts fit in a budget of 61680.0, in order.
pub fn flow_log_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 61680.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn flow_log_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Puts a flow log score code in its canonical FLS- form.
pub fn normalize_flow_log_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FLS-") {
        return clean.clone();
    }
    format!("FLS-{}", clean.replace(" ", "-"))
}

/// Names the band a flow log score score falls in.
pub fn classify_flow_log_score(score: f64) -> String {
    if score < 55.7 {
        return "elevated".to_string();
    }
    if score < 99.1 {
        return "high".to_string();
    }
    "ok".to_string()
}
