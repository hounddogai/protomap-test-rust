use super::*;

pub fn lenient_health_check_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Puts a lenient health check rule code in its canonical LHCR- form.
pub fn normalize_lenient_health_check_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LHCR-") {
        return clean.clone();
    }
    format!("LHCR-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 11707.0, in order.
pub fn lenient_health_check_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 11707.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a lenient health check rule score falls in.
pub fn classify_lenient_health_check_rule(score: f64) -> String {
    if score < 24.1 {
        return "low".to_string();
    }
    if score < 43.0 {
        return "ok".to_string();
    }
    if score < 61.2 {
        return "watch".to_string();
    }
    "medium".to_string()
}
