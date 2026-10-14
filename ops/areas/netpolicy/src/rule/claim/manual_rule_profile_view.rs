use super::*;

use acme_kit::text;

/// How screens and exports show a manual rule profile.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleProfileView {
    pub code_text: String,
    pub title: String,
    pub channel: String,
    pub score: String,
    pub margin_text: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_manual_rule_profile_view(item: &ManualRuleProfile) -> ManualRuleProfileView {
    ManualRuleProfileView {
        code_text: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 64),
        channel: item.channel.clone(),
        score: format!("{:.2}", item.score),
        margin_text: format!("{:.2}", item.margin),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_rule_profile_views(items: &[ManualRuleProfile]) -> Vec<ManualRuleProfileView> {
    let mut views: Vec<ManualRuleProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_rule_profile_view(item));
    }
    views
}

pub fn sum_manual_rule_profile_view(view: &ManualRuleProfileView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_score + parsed_margin)
}
