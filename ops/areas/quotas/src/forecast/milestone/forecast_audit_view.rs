use super::*;

use acme_kit::text;

/// How screens and exports show a forecast audit.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAuditView {
    pub code_text: String,
    pub name_text: String,
    pub summary: String,
    pub score: String,
    pub label_text: String,
    pub tenant_id: String,
    pub unit: String,
    pub segment: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_forecast_audit_view(item: &ForecastAudit) -> ForecastAuditView {
    ForecastAuditView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        summary: item.summary.clone(),
        score: format!("{:.2}", item.score),
        label_text: text::truncate(&item.label, 120),
        tenant_id: item.tenant_id.clone(),
        unit: text::truncate(&item.unit, 64),
        segment: item.segment.clone(),
        title: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_audit_views(items: &[ForecastAudit]) -> Vec<ForecastAuditView> {
    let mut views: Vec<ForecastAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_audit_view(item));
    }
    views
}

pub fn sum_forecast_audit_view(view: &ForecastAuditView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}
