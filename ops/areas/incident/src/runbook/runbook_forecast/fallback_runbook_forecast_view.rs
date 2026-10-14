use super::*;

use acme_kit::text;

/// How screens and exports show a fallback runbook forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRunbookForecastView {
    pub code_text: String,
    pub title: String,
    pub starts_at_text: String,
    pub reviewer: String,
    pub revision: String,
    pub namespace: String,
    pub channel: String,
    pub tenant_id: String,
    pub margin: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_fallback_runbook_forecast_view(item: &FallbackRunbookForecast) -> FallbackRunbookForecastView {
    FallbackRunbookForecastView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        starts_at_text: item.starts_at.to_string(),
        reviewer: text::truncate(&item.reviewer, 80),
        revision: item.revision.to_string(),
        namespace: text::truncate(&item.namespace, 64),
        channel: text::truncate(&item.channel, 120),
        tenant_id: item.tenant_id.clone(),
        margin: format!("{:.2}", item.margin),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_runbook_forecast_views(items: &[FallbackRunbookForecast]) -> Vec<FallbackRunbookForecastView> {
    let mut views: Vec<FallbackRunbookForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_runbook_forecast_view(item));
    }
    views
}

pub fn sum_fallback_runbook_forecast_view(view: &FallbackRunbookForecastView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_revision as f64) + parsed_margin)
}
