use super::*;

/// How screens and exports show a lenient reservation allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationAllocationView {
    pub code_text: String,
    pub name: String,
    pub maximum_text: String,
    pub currency_text: String,
    pub scope: String,
    pub active_text: String,
    pub attempts: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_lenient_reservation_allocation_view(item: &LenientReservationAllocation) -> LenientReservationAllocationView {
    LenientReservationAllocationView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        currency_text: item.currency.clone(),
        scope: item.scope.clone(),
        active_text: format!("{}", item.active),
        attempts: item.attempts.to_string(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_reservation_allocation_views(
    items: &[LenientReservationAllocation],
) -> Vec<LenientReservationAllocationView> {
    let mut views: Vec<LenientReservationAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_reservation_allocation_view(item));
    }
    views
}

pub fn sum_lenient_reservation_allocation_view(view: &LenientReservationAllocationView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_maximum + (parsed_attempts as f64) + (parsed_grace_days as f64))
}
