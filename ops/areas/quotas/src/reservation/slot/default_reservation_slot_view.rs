use super::*;

use acme_kit::text;

/// How screens and exports show a default reservation slot.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationSlotView {
    pub code: String,
    pub name_text: String,
    pub namespace: String,
    pub batch_size: String,
    pub category: String,
    pub notes_text: String,
    pub approved: String,
    pub cipher_text: String,
    pub pinned: String,
    pub cost: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_default_reservation_slot_view(item: &DefaultReservationSlot) -> DefaultReservationSlotView {
    DefaultReservationSlotView {
        code: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        namespace: text::truncate(&item.namespace, 64),
        batch_size: item.batch_size.to_string(),
        category: item.category.clone(),
        notes_text: item.notes.clone(),
        approved: format!("{}", item.approved),
        cipher_text: item.cipher.clone(),
        pinned: format!("{}", item.pinned),
        cost: format!("{:.2}", item.cost),
        cluster: text::truncate(&item.cluster, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_reservation_slot_views(items: &[DefaultReservationSlot]) -> Vec<DefaultReservationSlotView> {
    let mut views: Vec<DefaultReservationSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_reservation_slot_view(item));
    }
    views
}

pub fn sum_default_reservation_slot_view(view: &DefaultReservationSlotView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_cost)
}
