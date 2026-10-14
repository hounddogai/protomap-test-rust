use super::*;

use acme_kit::text;

/// How screens and exports show a timeline schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineScheduleView {
    pub code: String,
    pub name_text: String,
    pub key_id: String,
    pub grace_days_text: String,
    pub amount: String,
    pub channel: String,
    pub unit_text: String,
    pub cost_text: String,
    pub expires_at: String,
    pub quota_units: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_timeline_schedule_view(item: &TimelineSchedule) -> TimelineScheduleView {
    TimelineScheduleView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        key_id: item.key_id.clone(),
        grace_days_text: item.grace_days.to_string(),
        amount: format!("{:.2}", item.amount),
        channel: item.channel.clone(),
        unit_text: text::truncate(&item.unit, 80),
        cost_text: format!("{:.2}", item.cost),
        expires_at: item.expires_at.to_string(),
        quota_units: item.quota_units.to_string(),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_schedule_views(items: &[TimelineSchedule]) -> Vec<TimelineScheduleView> {
    let mut views: Vec<TimelineScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_schedule_view(item));
    }
    views
}

pub fn sum_timeline_schedule_view(view: &TimelineScheduleView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_amount + parsed_cost)
}
