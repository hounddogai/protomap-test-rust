use super::*;

/// How screens and exports show a identity rule.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRuleView {
    pub code: String,
    pub label_text: String,
    pub principal: String,
    pub revision_text: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_identity_rule_view(item: &IdentityRule) -> IdentityRuleView {
    IdentityRuleView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        principal: item.principal.clone(),
        revision_text: item.revision.to_string(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_rule_views(items: &[IdentityRule]) -> Vec<IdentityRuleView> {
    let mut views: Vec<IdentityRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_rule_view(item));
    }
    views
}

pub fn sum_identity_rule_view(view: &IdentityRuleView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_priority as f64))
}
