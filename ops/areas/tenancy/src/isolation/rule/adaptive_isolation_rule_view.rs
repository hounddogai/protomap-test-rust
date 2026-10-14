use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive isolation rule.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIsolationRuleView {
    pub code_text: String,
    pub name_text: String,
    pub issuer_text: String,
    pub visible_text: String,
    pub burst_text: String,
    pub status_label: String,
}

pub fn to_adaptive_isolation_rule_view(item: &AdaptiveIsolationRule) -> AdaptiveIsolationRuleView {
    AdaptiveIsolationRuleView {
        code_text: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        issuer_text: item.issuer.clone(),
        visible_text: format!("{}", item.visible),
        burst_text: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_isolation_rule_views(items: &[AdaptiveIsolationRule]) -> Vec<AdaptiveIsolationRuleView> {
    let mut views: Vec<AdaptiveIsolationRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_isolation_rule_view(item));
    }
    views
}

pub fn sum_adaptive_isolation_rule_view(view: &AdaptiveIsolationRuleView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_burst as f64)
}
