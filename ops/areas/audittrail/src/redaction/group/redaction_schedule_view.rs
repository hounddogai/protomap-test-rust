use super::*;

use acme_kit::text;

/// How screens and exports show a redaction schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionScheduleView {
    pub code: String,
    pub label: String,
    pub cipher_text: String,
    pub unit: String,
    pub port: String,
    pub score: String,
    pub budget_text: String,
    pub region_text: String,
    pub status_label: String,
}

pub fn to_redaction_schedule_view(item: &RedactionSchedule) -> RedactionScheduleView {
    RedactionScheduleView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        cipher_text: item.cipher.clone(),
        unit: item.unit.clone(),
        port: item.port.to_string(),
        score: format!("{:.2}", item.score),
        budget_text: format!("{:.2}", item.budget),
        region_text: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_schedule_views(items: &[RedactionSchedule]) -> Vec<RedactionScheduleView> {
    let mut views: Vec<RedactionScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_schedule_view(item));
    }
    views
}

pub fn sum_redaction_schedule_view(view: &RedactionScheduleView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_port as f64 + parsed_score + parsed_budget)
}
