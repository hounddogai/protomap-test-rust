use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled dashboard record.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDashboardRecordView {
    pub code: String,
    pub label: String,
    pub approved: String,
    pub cluster: String,
    pub phone: String,
    pub quantity_text: String,
    pub limit: String,
    pub country: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_scheduled_dashboard_record_view(item: &ScheduledDashboardRecord) -> ScheduledDashboardRecordView {
    ScheduledDashboardRecordView {
        code: item.code.clone(),
        label: item.label.clone(),
        approved: format!("{}", item.approved),
        cluster: text::truncate(&item.cluster, 120),
        phone: item.phone.clone(),
        quantity_text: item.quantity.to_string(),
        limit: item.limit.to_string(),
        country: text::truncate(&item.country, 120),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_dashboard_record_views(items: &[ScheduledDashboardRecord]) -> Vec<ScheduledDashboardRecordView> {
    let mut views: Vec<ScheduledDashboardRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_dashboard_record_view(item));
    }
    views
}

pub fn sum_scheduled_dashboard_record_view(view: &ScheduledDashboardRecordView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_limit as f64) + (parsed_batch_size as f64))
}
