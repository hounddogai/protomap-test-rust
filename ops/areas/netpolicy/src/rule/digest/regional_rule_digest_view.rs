use super::*;

use acme_kit::text;

/// How screens and exports show a regional rule digest.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleDigestView {
    pub code: String,
    pub name_text: String,
    pub audience_text: String,
    pub priority_text: String,
    pub discount: String,
    pub ttl_seconds: String,
    pub description: String,
    pub reviewer: String,
    pub ceiling: String,
    pub weight: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_regional_rule_digest_view(item: &RegionalRuleDigest) -> RegionalRuleDigestView {
    RegionalRuleDigestView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        audience_text: item.audience.clone(),
        priority_text: item.priority.to_string(),
        discount: format!("{:.2}", item.discount),
        ttl_seconds: item.ttl_seconds.to_string(),
        description: item.description.clone(),
        reviewer: text::truncate(&item.reviewer, 120),
        ceiling: format!("{:.2}", item.ceiling),
        weight: format!("{:.2}", item.weight),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_rule_digest_views(items: &[RegionalRuleDigest]) -> Vec<RegionalRuleDigestView> {
    let mut views: Vec<RegionalRuleDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_rule_digest_view(item));
    }
    views
}

pub fn sum_regional_rule_digest_view(view: &RegionalRuleDigestView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_priority as f64 + parsed_discount + (parsed_ttl_seconds as f64))
}
