use super::*;

/// How screens and exports show a adaptive identity schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityScheduleView {
    pub code: String,
    pub title: String,
    pub approved: String,
    pub issuer: String,
    pub sequence: String,
    pub phone_text: String,
    pub port: String,
    pub grace_days_text: String,
    pub revision: String,
    pub weight_text: String,
    pub status_label: String,
}

pub fn to_adaptive_identity_schedule_view(item: &AdaptiveIdentitySchedule) -> AdaptiveIdentityScheduleView {
    AdaptiveIdentityScheduleView {
        code: item.code.clone(),
        title: item.title.clone(),
        approved: format!("{}", item.approved),
        issuer: item.issuer.clone(),
        sequence: item.sequence.to_string(),
        phone_text: item.phone.clone(),
        port: item.port.to_string(),
        grace_days_text: item.grace_days.to_string(),
        revision: item.revision.to_string(),
        weight_text: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_identity_schedule_views(items: &[AdaptiveIdentitySchedule]) -> Vec<AdaptiveIdentityScheduleView> {
    let mut views: Vec<AdaptiveIdentityScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_identity_schedule_view(item));
    }
    views
}

pub fn sum_adaptive_identity_schedule_view(view: &AdaptiveIdentityScheduleView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_port as f64) + (parsed_grace_days as f64))
}
