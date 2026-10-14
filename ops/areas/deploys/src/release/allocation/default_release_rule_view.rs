use super::*;

use acme_kit::text;

/// How screens and exports show a default release rule.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseRuleView {
    pub code_text: String,
    pub name_text: String,
    pub expires_at: String,
    pub scope: String,
    pub reviewer: String,
    pub segment_text: String,
    pub status_label: String,
}

pub fn to_default_release_rule_view(item: &DefaultReleaseRule) -> DefaultReleaseRuleView {
    DefaultReleaseRuleView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        expires_at: item.expires_at.to_string(),
        scope: item.scope.clone(),
        reviewer: item.reviewer.clone(),
        segment_text: text::truncate(&item.segment, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_release_rule_views(items: &[DefaultReleaseRule]) -> Vec<DefaultReleaseRuleView> {
    let mut views: Vec<DefaultReleaseRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_release_rule_view(item));
    }
    views
}

pub fn sum_default_release_rule_view(view: &DefaultReleaseRuleView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_expires_at as f64)
}
