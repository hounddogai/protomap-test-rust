use super::*;

/// How screens and exports show a silence metric.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceMetricView {
    pub code: String,
    pub label_text: String,
    pub attempts_text: String,
    pub summary: String,
    pub issuer: String,
    pub principal_text: String,
    pub unit: String,
    pub quota_units: String,
    pub ends_at_text: String,
    pub status_label: String,
}

pub fn to_silence_metric_view(item: &SilenceMetric) -> SilenceMetricView {
    SilenceMetricView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        attempts_text: item.attempts.to_string(),
        summary: item.summary.clone(),
        issuer: item.issuer.clone(),
        principal_text: item.principal.clone(),
        unit: item.unit.clone(),
        quota_units: item.quota_units.to_string(),
        ends_at_text: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_metric_views(items: &[SilenceMetric]) -> Vec<SilenceMetricView> {
    let mut views: Vec<SilenceMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_metric_view(item));
    }
    views
}

pub fn sum_silence_metric_view(view: &SilenceMetricView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_quota_units as f64) + (parsed_ends_at as f64))
}
