use super::*;

/// How screens and exports show a regional rule tier.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleTierView {
    pub code: String,
    pub title: String,
    pub expires_at: String,
    pub namespace: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_regional_rule_tier_view(item: &RegionalRuleTier) -> RegionalRuleTierView {
    RegionalRuleTierView {
        code: item.code.clone(),
        title: item.title.clone(),
        expires_at: item.expires_at.to_string(),
        namespace: item.namespace.clone(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_rule_tier_views(items: &[RegionalRuleTier]) -> Vec<RegionalRuleTierView> {
    let mut views: Vec<RegionalRuleTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_rule_tier_view(item));
    }
    views
}

pub fn sum_regional_rule_tier_view(view: &RegionalRuleTierView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_ratio)
}
