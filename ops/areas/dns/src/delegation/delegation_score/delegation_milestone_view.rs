use super::*;

use acme_kit::text;

/// How screens and exports show a delegation milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMilestoneView {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub contact_email: String,
    pub active_text: String,
    pub priority: String,
    pub key_id: String,
    pub minimum_text: String,
    pub discount_text: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_delegation_milestone_view(item: &DelegationMilestone) -> DelegationMilestoneView {
    DelegationMilestoneView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        tenant_id: text::truncate(&item.tenant_id, 64),
        contact_email: item.contact_email.clone(),
        active_text: format!("{}", item.active),
        priority: item.priority.to_string(),
        key_id: item.key_id.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        discount_text: format!("{:.2}", item.discount),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_milestone_views(items: &[DelegationMilestone]) -> Vec<DelegationMilestoneView> {
    let mut views: Vec<DelegationMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_milestone_view(item));
    }
    views
}

pub fn sum_delegation_milestone_view(view: &DelegationMilestoneView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_priority as f64 + parsed_minimum + parsed_discount)
}
