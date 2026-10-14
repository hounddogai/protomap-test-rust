use super::*;

use acme_kit::text;

/// How screens and exports show a reservation allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAllocationView {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub host: String,
    pub fingerprint_text: String,
    pub amount_text: String,
    pub unit_text: String,
    pub limit: String,
    pub cluster_text: String,
    pub status_label: String,
}

pub fn to_reservation_allocation_view(item: &ReservationAllocation) -> ReservationAllocationView {
    ReservationAllocationView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        tenant_id: item.tenant_id.clone(),
        host: item.host.clone(),
        fingerprint_text: item.fingerprint.clone(),
        amount_text: format!("{:.2}", item.amount),
        unit_text: item.unit.clone(),
        limit: item.limit.to_string(),
        cluster_text: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_allocation_views(items: &[ReservationAllocation]) -> Vec<ReservationAllocationView> {
    let mut views: Vec<ReservationAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_allocation_view(item));
    }
    views
}

pub fn sum_reservation_allocation_view(view: &ReservationAllocationView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_amount + (parsed_limit as f64))
}
