use super::*;

use acme_kit::text;

/// How screens and exports show a fallback alert rule.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAlertRuleView {
    pub code: String,
    pub name: String,
    pub expires_at: String,
    pub reviewer_text: String,
    pub updated_at: String,
    pub channel_text: String,
    pub status_label: String,
}

pub fn to_fallback_alert_rule_view(item: &FallbackAlertRule) -> FallbackAlertRuleView {
    FallbackAlertRuleView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        expires_at: item.expires_at.to_string(),
        reviewer_text: text::truncate(&item.reviewer, 40),
        updated_at: item.updated_at.to_string(),
        channel_text: text::truncate(&item.channel, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_alert_rule_views(items: &[FallbackAlertRule]) -> Vec<FallbackAlertRuleView> {
    let mut views: Vec<FallbackAlertRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_alert_rule_view(item));
    }
    views
}

pub fn sum_fallback_alert_rule_view(view: &FallbackAlertRuleView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_updated_at as f64))
}
