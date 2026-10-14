use super::*;

use acme_kit::text;

/// How screens and exports show a forecast draft.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastDraftView {
    pub code_text: String,
    pub name: String,
    pub quota_units: String,
    pub updated_at: String,
    pub archived: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_forecast_draft_view(item: &ForecastDraft) -> ForecastDraftView {
    ForecastDraftView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 120),
        quota_units: item.quota_units.to_string(),
        updated_at: item.updated_at.to_string(),
        archived: format!("{}", item.archived),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_draft_views(items: &[ForecastDraft]) -> Vec<ForecastDraftView> {
    let mut views: Vec<ForecastDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_draft_view(item));
    }
    views
}

pub fn sum_forecast_draft_view(view: &ForecastDraftView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_updated_at as f64) + (parsed_created_at as f64))
}
