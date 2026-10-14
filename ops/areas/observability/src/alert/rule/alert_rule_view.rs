use super::*;

use acme_kit::text;

/// How screens and exports show a alert rule.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertRuleView {
    pub code: String,
    pub title_text: String,
    pub archived: String,
    pub maximum: String,
    pub window_days_text: String,
    pub retries: String,
    pub updated_at: String,
    pub rotation_days: String,
    pub external_ref: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_alert_rule_view(item: &AlertRule) -> AlertRuleView {
    AlertRuleView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        archived: format!("{}", item.archived),
        maximum: format!("{:.2}", item.maximum),
        window_days_text: item.window_days.to_string(),
        retries: item.retries.to_string(),
        updated_at: item.updated_at.to_string(),
        rotation_days: item.rotation_days.to_string(),
        external_ref: text::truncate(&item.external_ref, 64),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_rule_views(items: &[AlertRule]) -> Vec<AlertRuleView> {
    let mut views: Vec<AlertRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_rule_view(item));
    }
    views
}

pub fn sum_alert_rule_view(view: &AlertRuleView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_maximum + (parsed_window_days as f64) + (parsed_retries as f64))
}
