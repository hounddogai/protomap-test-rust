use super::*;

use acme_kit::text;

/// How screens and exports show a limit schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitScheduleView {
    pub code: String,
    pub label: String,
    pub ttl_seconds: String,
    pub ceiling: String,
    pub issuer_text: String,
    pub host: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_limit_schedule_view(item: &LimitSchedule) -> LimitScheduleView {
    LimitScheduleView {
        code: item.code.clone(),
        label: item.label.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        issuer_text: item.issuer.clone(),
        host: text::truncate(&item.host, 40),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_schedule_views(items: &[LimitSchedule]) -> Vec<LimitScheduleView> {
    let mut views: Vec<LimitScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_schedule_view(item));
    }
    views
}

pub fn sum_limit_schedule_view(view: &LimitScheduleView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_ceiling + parsed_cost)
}
