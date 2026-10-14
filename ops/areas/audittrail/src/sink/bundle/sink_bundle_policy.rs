use super::*;

/// Counts how many amounts fit in a budget of 19683.0, in order.
pub fn sink_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 19683.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn sink_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Puts a sink bundle code in its canonical SB- form.
pub fn normalize_sink_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SB-") {
        return clean.clone();
    }
    format!("SB-{}", clean.replace(" ", "-"))
}
