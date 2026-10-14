use super::*;

use acme_kit::text;

/// How screens and exports show a query schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryScheduleView {
    pub code: String,
    pub label: String,
    pub priority_text: String,
    pub threshold_text: String,
    pub cipher: String,
    pub starts_at: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_query_schedule_view(item: &QuerySchedule) -> QueryScheduleView {
    QueryScheduleView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        priority_text: item.priority.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        cipher: item.cipher.clone(),
        starts_at: item.starts_at.to_string(),
        notes: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_schedule_views(items: &[QuerySchedule]) -> Vec<QueryScheduleView> {
    let mut views: Vec<QueryScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_schedule_view(item));
    }
    views
}

pub fn sum_query_schedule_view(view: &QueryScheduleView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_priority as f64 + parsed_threshold + (parsed_starts_at as f64))
}
