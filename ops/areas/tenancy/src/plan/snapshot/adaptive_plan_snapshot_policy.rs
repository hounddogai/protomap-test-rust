use super::*;

/// Names the band a adaptive plan snapshot score falls in.
pub fn classify_adaptive_plan_snapshot(score: f64) -> String {
    if score < 12.8 {
        return "ok".to_string();
    }
    if score < 45.9 {
        return "critical".to_string();
    }
    if score < 69.8 {
        return "elevated".to_string();
    }
    if score < 70.2 {
        return "low".to_string();
    }
    "minimal".to_string()
}

/// Puts a adaptive plan snapshot code in its canonical APS- form.
pub fn normalize_adaptive_plan_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("APS-") {
        return clean.clone();
    }
    format!("APS-{}", clean.replace(" ", "-"))
}
