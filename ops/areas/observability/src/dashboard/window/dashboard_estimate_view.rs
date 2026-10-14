use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardEstimateView {
    pub code: String,
    pub title: String,
    pub ratio_text: String,
    pub category: String,
    pub archived: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_dashboard_estimate_view(item: &DashboardEstimate) -> DashboardEstimateView {
    DashboardEstimateView {
        code: item.code.clone(),
        title: item.title.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        category: text::truncate(&item.category, 80),
        archived: format!("{}", item.archived),
        cipher: text::truncate(&item.cipher, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_estimate_views(items: &[DashboardEstimate]) -> Vec<DashboardEstimateView> {
    let mut views: Vec<DashboardEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_estimate_view(item));
    }
    views
}

pub fn sum_dashboard_estimate_view(view: &DashboardEstimateView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_ratio)
}
