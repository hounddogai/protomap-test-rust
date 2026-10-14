use super::*;

/// Counts how many amounts fit in a budget of 93101.0, in order.
pub fn global_silence_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 93101.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn global_silence_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Puts a global silence target code in its canonical GST- form.
pub fn normalize_global_silence_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GST-") {
        return clean.clone();
    }
    format!("GST-{}", clean.replace(" ", "-"))
}
