use super::*;

/// How screens and exports show a rule tier.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleTierView {
    pub code: String,
    pub label_text: String,
    pub active: String,
    pub weight: String,
    pub cluster: String,
    pub issuer: String,
    pub retries: String,
    pub key_id: String,
    pub ratio_text: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_rule_tier_view(item: &RuleTier) -> RuleTierView {
    RuleTierView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        active: format!("{}", item.active),
        weight: format!("{:.2}", item.weight),
        cluster: item.cluster.clone(),
        issuer: item.issuer.clone(),
        retries: item.retries.to_string(),
        key_id: item.key_id.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_tier_views(items: &[RuleTier]) -> Vec<RuleTierView> {
    let mut views: Vec<RuleTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_tier_view(item));
    }
    views
}

pub fn sum_rule_tier_view(view: &RuleTierView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_weight + (parsed_retries as f64) + parsed_ratio)
}
