use super::*;

use acme_kit::text;

/// How screens and exports show a global health check allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalHealthCheckAllocationView {
    pub code: String,
    pub title: String,
    pub phone_text: String,
    pub contact_email: String,
    pub port: String,
    pub capacity: String,
    pub updated_at: String,
    pub archived_text: String,
    pub status_label: String,
}

pub fn to_global_health_check_allocation_view(item: &GlobalHealthCheckAllocation) -> GlobalHealthCheckAllocationView {
    GlobalHealthCheckAllocationView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        phone_text: item.phone.clone(),
        contact_email: text::truncate(&item.contact_email, 80),
        port: item.port.to_string(),
        capacity: item.capacity.to_string(),
        updated_at: item.updated_at.to_string(),
        archived_text: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_health_check_allocation_views(
    items: &[GlobalHealthCheckAllocation],
) -> Vec<GlobalHealthCheckAllocationView> {
    let mut views: Vec<GlobalHealthCheckAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_health_check_allocation_view(item));
    }
    views
}

pub fn sum_global_health_check_allocation_view(view: &GlobalHealthCheckAllocationView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_port as f64 + (parsed_capacity as f64) + (parsed_updated_at as f64))
}
