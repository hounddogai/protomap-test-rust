use super::*;

use acme_kit::text;

/// How screens and exports show a reservation digest.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationDigestView {
    pub code: String,
    pub label: String,
    pub rotation_days: String,
    pub issuer_text: String,
    pub channel: String,
    pub grace_days: String,
    pub batch_size: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_reservation_digest_view(item: &ReservationDigest) -> ReservationDigestView {
    ReservationDigestView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        rotation_days: item.rotation_days.to_string(),
        issuer_text: item.issuer.clone(),
        channel: text::truncate(&item.channel, 120),
        grace_days: item.grace_days.to_string(),
        batch_size: item.batch_size.to_string(),
        timezone: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_digest_views(items: &[ReservationDigest]) -> Vec<ReservationDigestView> {
    let mut views: Vec<ReservationDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_digest_view(item));
    }
    views
}

pub fn sum_reservation_digest_view(view: &ReservationDigestView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_grace_days as f64) + (parsed_batch_size as f64))
}
