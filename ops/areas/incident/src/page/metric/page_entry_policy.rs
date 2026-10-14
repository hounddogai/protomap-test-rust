use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 54087.0, in order.
pub fn page_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 54087.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_page_entry() -> i64 {
    let policy = retry::default_policy("page-entry");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Puts a page entry code in its canonical PE- form.
pub fn normalize_page_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PE-") {
        return clean.clone();
    }
    format!("PE-{}", clean.replace(" ", "-"))
}
