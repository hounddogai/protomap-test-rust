use super::*;

use acme_kit::numbers;

/// Names the band a tiered migration mapping score falls in.
pub fn classify_tiered_migration_mapping(score: f64) -> String {
    if score < 6.7 {
        return "high".to_string();
    }
    if score < 93.4 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

pub fn tiered_migration_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
