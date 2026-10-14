use super::*;

use acme_kit::text;

/// How screens and exports show a pending nonce target.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceTargetView {
    pub code: String,
    pub name: String,
    pub floor_value: String,
    pub pinned: String,
    pub key_id: String,
    pub currency: String,
    pub tenant_id: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_pending_nonce_target_view(item: &PendingNonceTarget) -> PendingNonceTargetView {
    PendingNonceTargetView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        floor_value: format!("{:.2}", item.floor_value),
        pinned: format!("{}", item.pinned),
        key_id: item.key_id.clone(),
        currency: item.currency.clone(),
        tenant_id: item.tenant_id.clone(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_nonce_target_views(items: &[PendingNonceTarget]) -> Vec<PendingNonceTargetView> {
    let mut views: Vec<PendingNonceTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_nonce_target_view(item));
    }
    views
}

pub fn sum_pending_nonce_target_view(view: &PendingNonceTargetView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_floor_value + (parsed_priority as f64))
}
