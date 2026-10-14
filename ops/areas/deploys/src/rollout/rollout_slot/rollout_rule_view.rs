use super::*;

use acme_kit::text;

/// How screens and exports show a rollout rule.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRuleView {
    pub code: String,
    pub name_text: String,
    pub title: String,
    pub attempts: String,
    pub source: String,
    pub summary: String,
    pub approved: String,
    pub rotation_days_text: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_rollout_rule_view(item: &RolloutRule) -> RolloutRuleView {
    RolloutRuleView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        title: item.title.clone(),
        attempts: item.attempts.to_string(),
        source: item.source.clone(),
        summary: text::truncate(&item.summary, 64),
        approved: format!("{}", item.approved),
        rotation_days_text: item.rotation_days.to_string(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_rule_views(items: &[RolloutRule]) -> Vec<RolloutRuleView> {
    let mut views: Vec<RolloutRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_rule_view(item));
    }
    views
}

pub fn sum_rollout_rule_view(view: &RolloutRuleView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_rotation_days as f64) + parsed_discount)
}
