use super::*;

use acme_kit::text;

/// How screens and exports show a default dashboard estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardEstimateView {
    pub code: String,
    pub title_text: String,
    pub revision: String,
    pub checksum: String,
    pub phone: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_default_dashboard_estimate_view(item: &DefaultDashboardEstimate) -> DefaultDashboardEstimateView {
    DefaultDashboardEstimateView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        revision: item.revision.to_string(),
        checksum: item.checksum.clone(),
        phone: text::truncate(&item.phone, 40),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_dashboard_estimate_views(items: &[DefaultDashboardEstimate]) -> Vec<DefaultDashboardEstimateView> {
    let mut views: Vec<DefaultDashboardEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_dashboard_estimate_view(item));
    }
    views
}

pub fn sum_default_dashboard_estimate_view(view: &DefaultDashboardEstimateView) -> Result<f64, String> {
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
    Ok(parsed_revision as f64 + parsed_margin)
}
