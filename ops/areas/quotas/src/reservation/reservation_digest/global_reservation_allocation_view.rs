use super::*;

/// How screens and exports show a global reservation allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationAllocationView {
    pub code: String,
    pub name: String,
    pub ends_at: String,
    pub score_text: String,
    pub rotation_days_text: String,
    pub status_label: String,
}

pub fn to_global_reservation_allocation_view(item: &GlobalReservationAllocation) -> GlobalReservationAllocationView {
    GlobalReservationAllocationView {
        code: item.code.clone(),
        name: item.name.clone(),
        ends_at: item.ends_at.to_string(),
        score_text: format!("{:.2}", item.score),
        rotation_days_text: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_reservation_allocation_views(
    items: &[GlobalReservationAllocation],
) -> Vec<GlobalReservationAllocationView> {
    let mut views: Vec<GlobalReservationAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_reservation_allocation_view(item));
    }
    views
}

pub fn sum_global_reservation_allocation_view(view: &GlobalReservationAllocationView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_score + (parsed_rotation_days as f64))
}
