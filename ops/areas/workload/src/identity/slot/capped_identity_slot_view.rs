use super::*;

/// How screens and exports show a capped identity slot.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedIdentitySlotView {
    pub code: String,
    pub title: String,
    pub ttl_seconds: String,
    pub floor_value_text: String,
    pub notes: String,
    pub category: String,
    pub threshold_text: String,
    pub external_ref: String,
    pub checksum_text: String,
    pub summary_text: String,
    pub status_label: String,
}

pub fn to_capped_identity_slot_view(item: &CappedIdentitySlot) -> CappedIdentitySlotView {
    CappedIdentitySlotView {
        code: item.code.clone(),
        title: item.title.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        notes: item.notes.clone(),
        category: item.category.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        external_ref: item.external_ref.clone(),
        checksum_text: item.checksum.clone(),
        summary_text: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_identity_slot_views(items: &[CappedIdentitySlot]) -> Vec<CappedIdentitySlotView> {
    let mut views: Vec<CappedIdentitySlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_identity_slot_view(item));
    }
    views
}

pub fn sum_capped_identity_slot_view(view: &CappedIdentitySlotView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_floor_value + parsed_threshold)
}
