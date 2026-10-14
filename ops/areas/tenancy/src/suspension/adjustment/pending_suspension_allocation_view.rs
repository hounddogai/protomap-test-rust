use super::*;

use acme_kit::text;

/// How screens and exports show a pending suspension allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSuspensionAllocationView {
    pub code_text: String,
    pub label: String,
    pub attempts: String,
    pub tenant_id_text: String,
    pub namespace: String,
    pub quota_units_text: String,
    pub principal: String,
    pub starts_at_text: String,
    pub discount: String,
    pub weight: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_pending_suspension_allocation_view(item: &PendingSuspensionAllocation) -> PendingSuspensionAllocationView {
    PendingSuspensionAllocationView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        attempts: item.attempts.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        namespace: item.namespace.clone(),
        quota_units_text: item.quota_units.to_string(),
        principal: text::truncate(&item.principal, 80),
        starts_at_text: item.starts_at.to_string(),
        discount: format!("{:.2}", item.discount),
        weight: format!("{:.2}", item.weight),
        contact_email: text::truncate(&item.contact_email, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_suspension_allocation_views(
    items: &[PendingSuspensionAllocation],
) -> Vec<PendingSuspensionAllocationView> {
    let mut views: Vec<PendingSuspensionAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_suspension_allocation_view(item));
    }
    views
}

pub fn sum_pending_suspension_allocation_view(view: &PendingSuspensionAllocationView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_quota_units as f64) + (parsed_starts_at as f64))
}
