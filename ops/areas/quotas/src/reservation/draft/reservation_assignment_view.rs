use super::*;

use acme_kit::text;

/// How screens and exports show a reservation assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAssignmentView {
    pub code: String,
    pub name: String,
    pub batch_size_text: String,
    pub updated_at_text: String,
    pub revision: String,
    pub label: String,
    pub region: String,
    pub burst_text: String,
    pub locale: String,
    pub priority_text: String,
    pub status_label: String,
}

pub fn to_reservation_assignment_view(item: &ReservationAssignment) -> ReservationAssignmentView {
    ReservationAssignmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        batch_size_text: item.batch_size.to_string(),
        updated_at_text: item.updated_at.to_string(),
        revision: item.revision.to_string(),
        label: text::truncate(&item.label, 80),
        region: item.region.clone(),
        burst_text: item.burst.to_string(),
        locale: item.locale.clone(),
        priority_text: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_assignment_views(items: &[ReservationAssignment]) -> Vec<ReservationAssignmentView> {
    let mut views: Vec<ReservationAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_assignment_view(item));
    }
    views
}

pub fn sum_reservation_assignment_view(view: &ReservationAssignmentView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_updated_at as f64) + (parsed_revision as f64))
}
