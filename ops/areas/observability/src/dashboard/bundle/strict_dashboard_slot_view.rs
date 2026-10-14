use super::*;

use acme_kit::text;

/// How screens and exports show a strict dashboard slot.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardSlotView {
    pub code_text: String,
    pub name: String,
    pub currency: String,
    pub namespace: String,
    pub unit: String,
    pub revision: String,
    pub rotation_days_text: String,
    pub checksum_text: String,
    pub quantity_text: String,
    pub budget: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_strict_dashboard_slot_view(item: &StrictDashboardSlot) -> StrictDashboardSlotView {
    StrictDashboardSlotView {
        code_text: text::truncate(&item.code, 80),
        name: item.name.clone(),
        currency: text::truncate(&item.currency, 80),
        namespace: item.namespace.clone(),
        unit: item.unit.clone(),
        revision: item.revision.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        checksum_text: item.checksum.clone(),
        quantity_text: item.quantity.to_string(),
        budget: format!("{:.2}", item.budget),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_dashboard_slot_views(items: &[StrictDashboardSlot]) -> Vec<StrictDashboardSlotView> {
    let mut views: Vec<StrictDashboardSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_dashboard_slot_view(item));
    }
    views
}

pub fn sum_strict_dashboard_slot_view(view: &StrictDashboardSlotView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_rotation_days as f64) + (parsed_quantity as f64))
}
