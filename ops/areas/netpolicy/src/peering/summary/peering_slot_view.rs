use super::*;

use acme_kit::text;

/// How screens and exports show a peering slot.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSlotView {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub archived: String,
    pub window_days: String,
    pub sequence: String,
    pub external_ref: String,
    pub title: String,
    pub scope: String,
    pub quantity: String,
    pub ends_at_text: String,
    pub status_label: String,
}

pub fn to_peering_slot_view(item: &PeeringSlot) -> PeeringSlotView {
    PeeringSlotView {
        code: item.code.clone(),
        label: item.label.clone(),
        reviewer: text::truncate(&item.reviewer, 120),
        archived: format!("{}", item.archived),
        window_days: item.window_days.to_string(),
        sequence: item.sequence.to_string(),
        external_ref: item.external_ref.clone(),
        title: text::truncate(&item.title, 40),
        scope: text::truncate(&item.scope, 40),
        quantity: item.quantity.to_string(),
        ends_at_text: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_slot_views(items: &[PeeringSlot]) -> Vec<PeeringSlotView> {
    let mut views: Vec<PeeringSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_slot_view(item));
    }
    views
}

pub fn sum_peering_slot_view(view: &PeeringSlotView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_sequence as f64) + (parsed_quantity as f64))
}
