use super::*;

use acme_kit::text;

/// How screens and exports show a reservation summary.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSummaryView {
    pub code: String,
    pub label: String,
    pub description: String,
    pub version: String,
    pub audience_text: String,
    pub limit: String,
    pub ends_at: String,
    pub notes_text: String,
    pub status_label: String,
}

pub fn to_reservation_summary_view(item: &ReservationSummary) -> ReservationSummaryView {
    ReservationSummaryView {
        code: item.code.clone(),
        label: item.label.clone(),
        description: item.description.clone(),
        version: item.version.to_string(),
        audience_text: text::truncate(&item.audience, 120),
        limit: item.limit.to_string(),
        ends_at: item.ends_at.to_string(),
        notes_text: text::truncate(&item.notes, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_summary_views(items: &[ReservationSummary]) -> Vec<ReservationSummaryView> {
    let mut views: Vec<ReservationSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_summary_view(item));
    }
    views
}

pub fn sum_reservation_summary_view(view: &ReservationSummaryView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_version as f64 + (parsed_limit as f64) + (parsed_ends_at as f64))
}
