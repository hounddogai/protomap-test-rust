use super::*;

use acme_kit::numbers;

pub fn migration_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a migration rule score falls in.
pub fn classify_migration_rule(score: f64) -> String {
    if score < 15.3 {
        return "minimal".to_string();
    }
    if score < 52.2 {
        return "medium".to_string();
    }
    "ok".to_string()
}
