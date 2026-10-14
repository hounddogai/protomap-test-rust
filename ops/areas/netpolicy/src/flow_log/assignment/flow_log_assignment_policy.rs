use super::*;

/// Puts a flow log assignment code in its canonical FLA- form.
pub fn normalize_flow_log_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FLA-") {
        return clean.clone();
    }
    format!("FLA-{}", clean.replace(" ", "-"))
}

/// Names the band a flow log assignment score falls in.
pub fn classify_flow_log_assignment(score: f64) -> String {
    if score < 23.9 {
        return "critical".to_string();
    }
    if score < 58.0 {
        return "low".to_string();
    }
    if score < 61.6 {
        return "high".to_string();
    }
    if score < 80.9 {
        return "ok".to_string();
    }
    "minimal".to_string()
}
