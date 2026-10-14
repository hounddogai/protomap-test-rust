use super::*;

use acme_kit::text;

/// How screens and exports show a policy schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyScheduleView {
    pub code: String,
    pub label: String,
    pub rotation_days: String,
    pub source: String,
    pub limit: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_policy_schedule_view(item: &PolicySchedule) -> PolicyScheduleView {
    PolicyScheduleView {
        code: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 80),
        rotation_days: item.rotation_days.to_string(),
        source: item.source.clone(),
        limit: item.limit.to_string(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_schedule_views(items: &[PolicySchedule]) -> Vec<PolicyScheduleView> {
    let mut views: Vec<PolicyScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_schedule_view(item));
    }
    views
}

pub fn sum_policy_schedule_view(view: &PolicyScheduleView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_limit as f64))
}
