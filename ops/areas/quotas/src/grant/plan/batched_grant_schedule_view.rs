use super::*;

use acme_kit::text;

/// How screens and exports show a batched grant schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantScheduleView {
    pub code: String,
    pub name_text: String,
    pub margin_text: String,
    pub title: String,
    pub contact_email_text: String,
    pub revision: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_batched_grant_schedule_view(item: &BatchedGrantSchedule) -> BatchedGrantScheduleView {
    BatchedGrantScheduleView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        margin_text: format!("{:.2}", item.margin),
        title: item.title.clone(),
        contact_email_text: text::truncate(&item.contact_email, 40),
        revision: item.revision.to_string(),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_grant_schedule_views(items: &[BatchedGrantSchedule]) -> Vec<BatchedGrantScheduleView> {
    let mut views: Vec<BatchedGrantScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_grant_schedule_view(item));
    }
    views
}

pub fn sum_batched_grant_schedule_view(view: &BatchedGrantScheduleView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_margin + (parsed_revision as f64))
}
