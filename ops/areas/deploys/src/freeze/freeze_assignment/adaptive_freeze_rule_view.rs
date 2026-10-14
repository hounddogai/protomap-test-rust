use super::*;

/// How screens and exports show a adaptive freeze rule.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeRuleView {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub ends_at_text: String,
    pub port: String,
    pub sequence_text: String,
    pub category: String,
    pub principal: String,
    pub capacity_text: String,
    pub status_label: String,
}

pub fn to_adaptive_freeze_rule_view(item: &AdaptiveFreezeRule) -> AdaptiveFreezeRuleView {
    AdaptiveFreezeRuleView {
        code: item.code.clone(),
        title: item.title.clone(),
        audience: item.audience.clone(),
        ends_at_text: item.ends_at.to_string(),
        port: item.port.to_string(),
        sequence_text: item.sequence.to_string(),
        category: item.category.clone(),
        principal: item.principal.clone(),
        capacity_text: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_freeze_rule_views(items: &[AdaptiveFreezeRule]) -> Vec<AdaptiveFreezeRuleView> {
    let mut views: Vec<AdaptiveFreezeRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_freeze_rule_view(item));
    }
    views
}

pub fn sum_adaptive_freeze_rule_view(view: &AdaptiveFreezeRuleView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_port as f64) + (parsed_sequence as f64))
}
