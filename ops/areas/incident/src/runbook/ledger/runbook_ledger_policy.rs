use super::*;

use acme_kit::numbers;

/// Puts a runbook ledger code in its canonical RL- form.
pub fn normalize_runbook_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RL-") {
        return clean.clone();
    }
    format!("RL-{}", clean.replace(" ", "-"))
}

pub fn runbook_ledger_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn runbook_ledger_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}
