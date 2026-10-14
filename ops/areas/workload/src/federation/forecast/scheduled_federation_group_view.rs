use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled federation group.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationGroupView {
    pub code: String,
    pub label: String,
    pub description: String,
    pub checksum: String,
    pub port: String,
    pub summary_text: String,
    pub title: String,
    pub category_text: String,
    pub rotation_days_text: String,
    pub contact_email_text: String,
    pub created_at_text: String,
    pub status_label: String,
}

pub fn to_scheduled_federation_group_view(item: &ScheduledFederationGroup) -> ScheduledFederationGroupView {
    ScheduledFederationGroupView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        description: item.description.clone(),
        checksum: item.checksum.clone(),
        port: item.port.to_string(),
        summary_text: item.summary.clone(),
        title: item.title.clone(),
        category_text: text::truncate(&item.category, 120),
        rotation_days_text: item.rotation_days.to_string(),
        contact_email_text: item.contact_email.clone(),
        created_at_text: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_federation_group_views(items: &[ScheduledFederationGroup]) -> Vec<ScheduledFederationGroupView> {
    let mut views: Vec<ScheduledFederationGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_federation_group_view(item));
    }
    views
}

pub fn sum_scheduled_federation_group_view(view: &ScheduledFederationGroupView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_port as f64 + (parsed_rotation_days as f64) + (parsed_created_at as f64))
}
