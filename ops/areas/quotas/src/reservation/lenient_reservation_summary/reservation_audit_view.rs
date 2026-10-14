use super::*;

use acme_kit::text;

/// How screens and exports show a reservation audit.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAuditView {
    pub code: String,
    pub title: String,
    pub cluster_text: String,
    pub cost_text: String,
    pub burst: String,
    pub name: String,
    pub rate_text: String,
    pub source: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_reservation_audit_view(item: &ReservationAudit) -> ReservationAuditView {
    ReservationAuditView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        cluster_text: item.cluster.clone(),
        cost_text: format!("{:.2}", item.cost),
        burst: item.burst.to_string(),
        name: item.name.clone(),
        rate_text: format!("{:.2}", item.rate),
        source: item.source.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_audit_views(items: &[ReservationAudit]) -> Vec<ReservationAuditView> {
    let mut views: Vec<ReservationAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_audit_view(item));
    }
    views
}

pub fn sum_reservation_audit_view(view: &ReservationAuditView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_cost + (parsed_burst as f64) + parsed_rate)
}
