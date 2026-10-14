use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled binding schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBindingScheduleView {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub locale: String,
    pub budget: String,
    pub threshold_text: String,
    pub cluster: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_scheduled_binding_schedule_view(item: &ScheduledBindingSchedule) -> ScheduledBindingScheduleView {
    ScheduledBindingScheduleView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        phone: item.phone.clone(),
        locale: text::truncate(&item.locale, 64),
        budget: format!("{:.2}", item.budget),
        threshold_text: format!("{:.2}", item.threshold),
        cluster: text::truncate(&item.cluster, 40),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_binding_schedule_views(items: &[ScheduledBindingSchedule]) -> Vec<ScheduledBindingScheduleView> {
    let mut views: Vec<ScheduledBindingScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_binding_schedule_view(item));
    }
    views
}

pub fn sum_scheduled_binding_schedule_view(view: &ScheduledBindingScheduleView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_budget + parsed_threshold + parsed_score)
}
