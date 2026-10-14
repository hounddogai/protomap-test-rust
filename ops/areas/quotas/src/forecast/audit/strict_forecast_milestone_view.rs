use super::*;

use acme_kit::text;

/// How screens and exports show a strict forecast milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastMilestoneView {
    pub code: String,
    pub name: String,
    pub region: String,
    pub tenant_id: String,
    pub archived: String,
    pub label: String,
    pub pinned: String,
    pub created_at: String,
    pub principal_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_strict_forecast_milestone_view(item: &StrictForecastMilestone) -> StrictForecastMilestoneView {
    StrictForecastMilestoneView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        region: item.region.clone(),
        tenant_id: item.tenant_id.clone(),
        archived: format!("{}", item.archived),
        label: item.label.clone(),
        pinned: format!("{}", item.pinned),
        created_at: item.created_at.to_string(),
        principal_text: item.principal.clone(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_forecast_milestone_views(items: &[StrictForecastMilestone]) -> Vec<StrictForecastMilestoneView> {
    let mut views: Vec<StrictForecastMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_forecast_milestone_view(item));
    }
    views
}

pub fn sum_strict_forecast_milestone_view(view: &StrictForecastMilestoneView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_created_at as f64)
}
