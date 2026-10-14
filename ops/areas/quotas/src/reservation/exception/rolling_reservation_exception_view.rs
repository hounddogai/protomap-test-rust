use super::*;

use acme_kit::text;

/// How screens and exports show a rolling reservation exception.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingReservationExceptionView {
    pub code: String,
    pub label_text: String,
    pub channel_text: String,
    pub locale: String,
    pub audience: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_rolling_reservation_exception_view(item: &RollingReservationException) -> RollingReservationExceptionView {
    RollingReservationExceptionView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        channel_text: text::truncate(&item.channel, 40),
        locale: item.locale.clone(),
        audience: text::truncate(&item.audience, 80),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_reservation_exception_views(
    items: &[RollingReservationException],
) -> Vec<RollingReservationExceptionView> {
    let mut views: Vec<RollingReservationExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_reservation_exception_view(item));
    }
    views
}

pub fn sum_rolling_reservation_exception_view(view: &RollingReservationExceptionView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_version as f64)
}
