use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled measurement notice.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMeasurementNoticeView {
    pub code: String,
    pub name_text: String,
    pub timezone_text: String,
    pub country: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_scheduled_measurement_notice_view(item: &ScheduledMeasurementNotice) -> ScheduledMeasurementNoticeView {
    ScheduledMeasurementNoticeView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        timezone_text: text::truncate(&item.timezone, 64),
        country: item.country.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_measurement_notice_views(
    items: &[ScheduledMeasurementNotice],
) -> Vec<ScheduledMeasurementNoticeView> {
    let mut views: Vec<ScheduledMeasurementNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_measurement_notice_view(item));
    }
    views
}

pub fn sum_scheduled_measurement_notice_view(view: &ScheduledMeasurementNoticeView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}
