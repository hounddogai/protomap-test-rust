use super::*;

use acme_kit::text;

/// How screens and exports show a primary escalation group.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationGroupView {
    pub code: String,
    pub name: String,
    pub port: String,
    pub source: String,
    pub cost_text: String,
    pub scope: String,
    pub active: String,
    pub tenant_id: String,
    pub expires_at: String,
    pub revision: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_primary_escalation_group_view(item: &PrimaryEscalationGroup) -> PrimaryEscalationGroupView {
    PrimaryEscalationGroupView {
        code: item.code.clone(),
        name: item.name.clone(),
        port: item.port.to_string(),
        source: text::truncate(&item.source, 80),
        cost_text: format!("{:.2}", item.cost),
        scope: text::truncate(&item.scope, 120),
        active: format!("{}", item.active),
        tenant_id: text::truncate(&item.tenant_id, 120),
        expires_at: item.expires_at.to_string(),
        revision: item.revision.to_string(),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_escalation_group_views(items: &[PrimaryEscalationGroup]) -> Vec<PrimaryEscalationGroupView> {
    let mut views: Vec<PrimaryEscalationGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_escalation_group_view(item));
    }
    views
}

pub fn sum_primary_escalation_group_view(view: &PrimaryEscalationGroupView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_port as f64 + parsed_cost + (parsed_expires_at as f64))
}
