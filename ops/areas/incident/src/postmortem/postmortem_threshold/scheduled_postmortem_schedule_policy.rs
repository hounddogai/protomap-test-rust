use super::*;

use acme_kit::numbers;

pub fn scheduled_postmortem_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.42;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 8692.0, in order.
pub fn scheduled_postmortem_schedule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 8692.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a scheduled postmortem schedule code in its canonical SPS- form.
pub fn normalize_scheduled_postmortem_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SPS-") {
        return clean.clone();
    }
    format!("SPS-{}", clean.replace(" ", "-"))
}

/// Names the band a scheduled postmortem schedule score falls in.
pub fn classify_scheduled_postmortem_schedule(score: f64) -> String {
    if score < 9.2 {
        return "medium".to_string();
    }
    if score < 66.3 {
        return "watch".to_string();
    }
    if score < 82.4 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}
