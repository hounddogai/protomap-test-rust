use super::*;

/// Puts a zone rollup code in its canonical ZR- form.
pub fn normalize_zone_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ZR-") {
        return clean.clone();
    }
    format!("ZR-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 93222.0, in order.
pub fn zone_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 93222.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a zone rollup score falls in.
pub fn classify_zone_rollup(score: f64) -> String {
    if score < 33.7 {
        return "critical".to_string();
    }
    if score < 67.4 {
        return "ok".to_string();
    }
    if score < 68.4 {
        return "high".to_string();
    }
    if score < 69.3 {
        return "minimal".to_string();
    }
    "watch".to_string()
}
