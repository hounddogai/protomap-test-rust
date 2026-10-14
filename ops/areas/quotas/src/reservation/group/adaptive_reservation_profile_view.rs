use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive reservation profile.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReservationProfileView {
    pub code: String,
    pub label_text: String,
    pub channel: String,
    pub ends_at: String,
    pub revision: String,
    pub score_text: String,
    pub duration_minutes_text: String,
    pub discount: String,
    pub capacity: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_adaptive_reservation_profile_view(item: &AdaptiveReservationProfile) -> AdaptiveReservationProfileView {
    AdaptiveReservationProfileView {
        code: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        channel: item.channel.clone(),
        ends_at: item.ends_at.to_string(),
        revision: item.revision.to_string(),
        score_text: format!("{:.2}", item.score),
        duration_minutes_text: item.duration_minutes.to_string(),
        discount: format!("{:.2}", item.discount),
        capacity: item.capacity.to_string(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_reservation_profile_views(
    items: &[AdaptiveReservationProfile],
) -> Vec<AdaptiveReservationProfileView> {
    let mut views: Vec<AdaptiveReservationProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_reservation_profile_view(item));
    }
    views
}

pub fn sum_adaptive_reservation_profile_view(view: &AdaptiveReservationProfileView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_revision as f64) + parsed_score)
}
