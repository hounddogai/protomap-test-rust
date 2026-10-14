use super::*;

use acme_kit::text;

/// How screens and exports show a capped slo slot.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloSlotView {
    pub code_text: String,
    pub title_text: String,
    pub revision: String,
    pub currency_text: String,
    pub quota_units: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_capped_slo_slot_view(item: &CappedSloSlot) -> CappedSloSlotView {
    CappedSloSlotView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 120),
        revision: item.revision.to_string(),
        currency_text: text::truncate(&item.currency, 64),
        quota_units: item.quota_units.to_string(),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_slo_slot_views(items: &[CappedSloSlot]) -> Vec<CappedSloSlotView> {
    let mut views: Vec<CappedSloSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_slo_slot_view(item));
    }
    views
}

pub fn sum_capped_slo_slot_view(view: &CappedSloSlotView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_quota_units as f64))
}
