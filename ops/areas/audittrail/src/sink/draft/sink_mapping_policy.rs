use super::*;

/// Names the band a sink mapping score falls in.
pub fn classify_sink_mapping(score: f64) -> String {
    if score < 4.1 {
        return "critical".to_string();
    }
    if score < 54.9 {
        return "minimal".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 69642.0, in order.
pub fn sink_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 69642.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a sink mapping code in its canonical SM- form.
pub fn normalize_sink_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SM-") {
        return clean.clone();
    }
    format!("SM-{}", clean.replace(" ", "-"))
}
