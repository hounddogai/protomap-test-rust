use super::*;

use acme_kit::text;

/// How screens and exports show a reservation policy.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPolicyView {
    pub code: String,
    pub name: String,
    pub category_text: String,
    pub archived_text: String,
    pub active: String,
    pub batch_size: String,
    pub port: String,
    pub priority: String,
    pub pinned: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_reservation_policy_view(item: &ReservationPolicy) -> ReservationPolicyView {
    ReservationPolicyView {
        code: item.code.clone(),
        name: item.name.clone(),
        category_text: item.category.clone(),
        archived_text: format!("{}", item.archived),
        active: format!("{}", item.active),
        batch_size: item.batch_size.to_string(),
        port: item.port.to_string(),
        priority: item.priority.to_string(),
        pinned: format!("{}", item.pinned),
        owner_id: text::truncate(&item.owner_id, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_policy_views(items: &[ReservationPolicy]) -> Vec<ReservationPolicyView> {
    let mut views: Vec<ReservationPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_policy_view(item));
    }
    views
}

pub fn sum_reservation_policy_view(view: &ReservationPolicyView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_port as f64) + (parsed_priority as f64))
}
