use super::*;

/// Puts a manual migration manifest code in its canonical MMM- form.
pub fn normalize_manual_migration_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MMM-") {
        return clean.clone();
    }
    format!("MMM-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 8831.0, in order.
pub fn manual_migration_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 8831.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn manual_migration_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Names the band a manual migration manifest score falls in.
pub fn classify_manual_migration_manifest(score: f64) -> String {
    if score < 10.4 {
        return "medium".to_string();
    }
    if score < 10.7 {
        return "elevated".to_string();
    }
    if score < 37.6 {
        return "high".to_string();
    }
    "ok".to_string()
}
