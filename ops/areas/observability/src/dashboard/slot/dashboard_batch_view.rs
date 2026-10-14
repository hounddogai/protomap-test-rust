use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard batch.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardBatchView {
    pub code: String,
    pub name: String,
    pub issuer_text: String,
    pub phone: String,
    pub label: String,
    pub active: String,
    pub unit_text: String,
    pub ratio_text: String,
    pub description_text: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_dashboard_batch_view(item: &DashboardBatch) -> DashboardBatchView {
    DashboardBatchView {
        code: item.code.clone(),
        name: item.name.clone(),
        issuer_text: item.issuer.clone(),
        phone: item.phone.clone(),
        label: text::truncate(&item.label, 120),
        active: format!("{}", item.active),
        unit_text: item.unit.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        description_text: item.description.clone(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_batch_views(items: &[DashboardBatch]) -> Vec<DashboardBatchView> {
    let mut views: Vec<DashboardBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_batch_view(item));
    }
    views
}

pub fn sum_dashboard_batch_view(view: &DashboardBatchView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_ratio + (parsed_priority as f64))
}
