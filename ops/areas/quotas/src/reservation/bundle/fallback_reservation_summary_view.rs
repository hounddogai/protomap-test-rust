use super::*;

use acme_kit::text;

/// How screens and exports show a fallback reservation summary.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationSummaryView {
    pub code: String,
    pub title: String,
    pub limit: String,
    pub capacity_text: String,
    pub locale_text: String,
    pub unit: String,
    pub score: String,
    pub visible_text: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_fallback_reservation_summary_view(item: &FallbackReservationSummary) -> FallbackReservationSummaryView {
    FallbackReservationSummaryView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        limit: item.limit.to_string(),
        capacity_text: item.capacity.to_string(),
        locale_text: item.locale.clone(),
        unit: text::truncate(&item.unit, 120),
        score: format!("{:.2}", item.score),
        visible_text: format!("{}", item.visible),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_reservation_summary_views(
    items: &[FallbackReservationSummary],
) -> Vec<FallbackReservationSummaryView> {
    let mut views: Vec<FallbackReservationSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_reservation_summary_view(item));
    }
    views
}

pub fn sum_fallback_reservation_summary_view(view: &FallbackReservationSummaryView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_capacity as f64) + parsed_score)
}
