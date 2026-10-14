use super::*;

/// How screens and exports show a scheduled delegation profile.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDelegationProfileView {
    pub code: String,
    pub title_text: String,
    pub sequence_text: String,
    pub port_text: String,
    pub phone: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_scheduled_delegation_profile_view(item: &ScheduledDelegationProfile) -> ScheduledDelegationProfileView {
    ScheduledDelegationProfileView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        sequence_text: item.sequence.to_string(),
        port_text: item.port.to_string(),
        phone: item.phone.clone(),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_delegation_profile_views(
    items: &[ScheduledDelegationProfile],
) -> Vec<ScheduledDelegationProfileView> {
    let mut views: Vec<ScheduledDelegationProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_delegation_profile_view(item));
    }
    views
}

pub fn sum_scheduled_delegation_profile_view(view: &ScheduledDelegationProfileView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_port as f64) + (parsed_created_at as f64))
}
