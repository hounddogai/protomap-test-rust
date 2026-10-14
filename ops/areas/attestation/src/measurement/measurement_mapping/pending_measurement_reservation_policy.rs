use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_measurement_reservation() -> i64 {
    let policy = retry::default_policy("pending-measurement-reservation");
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

/// Puts a pending measurement reservation code in its canonical PMR- form.
pub fn normalize_pending_measurement_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PMR-") {
        return clean.clone();
    }
    format!("PMR-{}", clean.replace(" ", "-"))
}
