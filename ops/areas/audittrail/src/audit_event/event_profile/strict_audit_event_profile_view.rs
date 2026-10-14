use super::*;

use acme_kit::text;

/// How screens and exports show a strict audit event profile.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventProfileView {
    pub code_text: String,
    pub name: String,
    pub description: String,
    pub channel_text: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_strict_audit_event_profile_view(item: &StrictAuditEventProfile) -> StrictAuditEventProfileView {
    StrictAuditEventProfileView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 80),
        description: item.description.clone(),
        channel_text: item.channel.clone(),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_audit_event_profile_views(items: &[StrictAuditEventProfile]) -> Vec<StrictAuditEventProfileView> {
    let mut views: Vec<StrictAuditEventProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_audit_event_profile_view(item));
    }
    views
}

pub fn sum_strict_audit_event_profile_view(view: &StrictAuditEventProfileView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
