use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled rule override.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleOverrideView {
    pub code: String,
    pub label: String,
    pub title: String,
    pub grace_days_text: String,
    pub amount: String,
    pub issuer: String,
    pub limit_text: String,
    pub weight: String,
    pub locale: String,
    pub source_text: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_scheduled_rule_override_view(item: &ScheduledRuleOverride) -> ScheduledRuleOverrideView {
    ScheduledRuleOverrideView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        title: text::truncate(&item.title, 40),
        grace_days_text: item.grace_days.to_string(),
        amount: format!("{:.2}", item.amount),
        issuer: item.issuer.clone(),
        limit_text: item.limit.to_string(),
        weight: format!("{:.2}", item.weight),
        locale: item.locale.clone(),
        source_text: item.source.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_rule_override_views(items: &[ScheduledRuleOverride]) -> Vec<ScheduledRuleOverrideView> {
    let mut views: Vec<ScheduledRuleOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_rule_override_view(item));
    }
    views
}

pub fn sum_scheduled_rule_override_view(view: &ScheduledRuleOverrideView) -> Result<f64, String> {
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
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_amount + (parsed_limit as f64))
}
