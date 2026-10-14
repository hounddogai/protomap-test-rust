use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal dashboard estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardEstimateView {
    pub code: String,
    pub label_text: String,
    pub ceiling: String,
    pub summary_text: String,
    pub port: String,
    pub region: String,
    pub description_text: String,
    pub budget: String,
    pub external_ref_text: String,
    pub status_label: String,
}

pub fn to_seasonal_dashboard_estimate_view(item: &SeasonalDashboardEstimate) -> SeasonalDashboardEstimateView {
    SeasonalDashboardEstimateView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        summary_text: text::truncate(&item.summary, 40),
        port: item.port.to_string(),
        region: item.region.clone(),
        description_text: item.description.clone(),
        budget: format!("{:.2}", item.budget),
        external_ref_text: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_dashboard_estimate_views(items: &[SeasonalDashboardEstimate]) -> Vec<SeasonalDashboardEstimateView> {
    let mut views: Vec<SeasonalDashboardEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_dashboard_estimate_view(item));
    }
    views
}

pub fn sum_seasonal_dashboard_estimate_view(view: &SeasonalDashboardEstimateView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_ceiling + (parsed_port as f64) + parsed_budget)
}
