use super::*;

use acme_kit::text;

/// How screens and exports show a rolling token schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenScheduleView {
    pub code: String,
    pub label_text: String,
    pub threshold_text: String,
    pub retries: String,
    pub host_text: String,
    pub burst: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_rolling_token_schedule_view(item: &RollingTokenSchedule) -> RollingTokenScheduleView {
    RollingTokenScheduleView {
        code: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        retries: item.retries.to_string(),
        host_text: item.host.clone(),
        burst: item.burst.to_string(),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_token_schedule_views(items: &[RollingTokenSchedule]) -> Vec<RollingTokenScheduleView> {
    let mut views: Vec<RollingTokenScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_token_schedule_view(item));
    }
    views
}

pub fn sum_rolling_token_schedule_view(view: &RollingTokenScheduleView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_threshold + (parsed_retries as f64) + (parsed_burst as f64))
}
