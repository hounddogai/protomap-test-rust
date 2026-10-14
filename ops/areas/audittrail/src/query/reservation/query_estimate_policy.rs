use super::*;

/// Counts how many amounts fit in a budget of 24464.0, in order.
pub fn query_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 24464.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a query estimate code in its canonical QE- form.
pub fn normalize_query_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QE-") {
        return clean.clone();
    }
    format!("QE-{}", clean.replace(" ", "-"))
}

/// Names the band a query estimate score falls in.
pub fn classify_query_estimate(score: f64) -> String {
    if score < 25.1 {
        return "minimal".to_string();
    }
    if score < 39.3 {
        return "low".to_string();
    }
    "medium".to_string()
}
