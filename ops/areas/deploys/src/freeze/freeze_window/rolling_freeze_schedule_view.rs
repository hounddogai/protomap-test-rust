use super::*;

/// How screens and exports show a rolling freeze schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFreezeScheduleView {
    pub code: String,
    pub title: String,
    pub country: String,
    pub version_text: String,
    pub pinned: String,
    pub locale_text: String,
    pub created_at_text: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_rolling_freeze_schedule_view(item: &RollingFreezeSchedule) -> RollingFreezeScheduleView {
    RollingFreezeScheduleView {
        code: item.code.clone(),
        title: item.title.clone(),
        country: item.country.clone(),
        version_text: item.version.to_string(),
        pinned: format!("{}", item.pinned),
        locale_text: item.locale.clone(),
        created_at_text: item.created_at.to_string(),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_freeze_schedule_views(items: &[RollingFreezeSchedule]) -> Vec<RollingFreezeScheduleView> {
    let mut views: Vec<RollingFreezeScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_freeze_schedule_view(item));
    }
    views
}

pub fn sum_rolling_freeze_schedule_view(view: &RollingFreezeScheduleView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_version as f64 + (parsed_created_at as f64) + parsed_amount)
}
