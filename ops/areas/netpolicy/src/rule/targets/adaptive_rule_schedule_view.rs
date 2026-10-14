use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive rule schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleScheduleView {
    pub code: String,
    pub title_text: String,
    pub summary: String,
    pub source: String,
    pub rotation_days: String,
    pub budget: String,
    pub grace_days: String,
    pub ceiling: String,
    pub quantity_text: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_adaptive_rule_schedule_view(item: &AdaptiveRuleSchedule) -> AdaptiveRuleScheduleView {
    AdaptiveRuleScheduleView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        summary: item.summary.clone(),
        source: text::truncate(&item.source, 120),
        rotation_days: item.rotation_days.to_string(),
        budget: format!("{:.2}", item.budget),
        grace_days: item.grace_days.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        quantity_text: item.quantity.to_string(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_rule_schedule_views(items: &[AdaptiveRuleSchedule]) -> Vec<AdaptiveRuleScheduleView> {
    let mut views: Vec<AdaptiveRuleScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_rule_schedule_view(item));
    }
    views
}

pub fn sum_adaptive_rule_schedule_view(view: &AdaptiveRuleScheduleView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_budget + (parsed_grace_days as f64))
}
