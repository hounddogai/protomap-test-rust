use super::*;

/// How screens and exports show a strict forecast allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastAllocationView {
    pub code_text: String,
    pub title: String,
    pub external_ref_text: String,
    pub reviewer: String,
    pub grace_days: String,
    pub margin: String,
    pub revision: String,
    pub sequence: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_strict_forecast_allocation_view(item: &StrictForecastAllocation) -> StrictForecastAllocationView {
    StrictForecastAllocationView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        external_ref_text: item.external_ref.clone(),
        reviewer: item.reviewer.clone(),
        grace_days: item.grace_days.to_string(),
        margin: format!("{:.2}", item.margin),
        revision: item.revision.to_string(),
        sequence: item.sequence.to_string(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_forecast_allocation_views(items: &[StrictForecastAllocation]) -> Vec<StrictForecastAllocationView> {
    let mut views: Vec<StrictForecastAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_forecast_allocation_view(item));
    }
    views
}

pub fn sum_strict_forecast_allocation_view(view: &StrictForecastAllocationView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_margin + (parsed_revision as f64))
}
