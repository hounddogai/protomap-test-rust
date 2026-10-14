use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled binding allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBindingAllocationView {
    pub code: String,
    pub label: String,
    pub description: String,
    pub approved: String,
    pub quota_units: String,
    pub phone: String,
    pub quantity: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_scheduled_binding_allocation_view(item: &ScheduledBindingAllocation) -> ScheduledBindingAllocationView {
    ScheduledBindingAllocationView {
        code: item.code.clone(),
        label: item.label.clone(),
        description: text::truncate(&item.description, 40),
        approved: format!("{}", item.approved),
        quota_units: item.quota_units.to_string(),
        phone: text::truncate(&item.phone, 64),
        quantity: item.quantity.to_string(),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_binding_allocation_views(
    items: &[ScheduledBindingAllocation],
) -> Vec<ScheduledBindingAllocationView> {
    let mut views: Vec<ScheduledBindingAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_binding_allocation_view(item));
    }
    views
}

pub fn sum_scheduled_binding_allocation_view(view: &ScheduledBindingAllocationView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_quantity as f64))
}
