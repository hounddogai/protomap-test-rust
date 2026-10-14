use super::*;

/// Counts how many amounts fit in a budget of 75823.0, in order.
pub fn capped_data_export_allocation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 75823.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a capped data export allocation score falls in.
pub fn classify_capped_data_export_allocation(score: f64) -> String {
    if score < 23.1 {
        return "critical".to_string();
    }
    if score < 58.4 {
        return "minimal".to_string();
    }
    "low".to_string()
}

/// Puts a capped data export allocation code in its canonical CDEA- form.
pub fn normalize_capped_data_export_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CDEA-") {
        return clean.clone();
    }
    format!("CDEA-{}", clean.replace(" ", "-"))
}
