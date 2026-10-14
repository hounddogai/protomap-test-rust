use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 69062.0, in order.
pub fn redaction_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 69062.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a redaction mapping code in its canonical RM- form.
pub fn normalize_redaction_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RM-") {
        return clean.clone();
    }
    format!("RM-{}", clean.replace(" ", "-"))
}

pub fn redaction_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a redaction mapping score falls in.
pub fn classify_redaction_mapping(score: f64) -> String {
    if score < 32.2 {
        return "ok".to_string();
    }
    if score < 70.9 {
        return "medium".to_string();
    }
    if score < 76.8 {
        return "low".to_string();
    }
    if score < 91.1 {
        return "high".to_string();
    }
    "minimal".to_string()
}
