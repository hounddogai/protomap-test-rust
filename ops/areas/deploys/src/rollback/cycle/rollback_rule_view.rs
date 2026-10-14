use super::*;

/// How screens and exports show a rollback rule.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRuleView {
    pub code: String,
    pub label_text: String,
    pub phone: String,
    pub sequence_text: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_rollback_rule_view(item: &RollbackRule) -> RollbackRuleView {
    RollbackRuleView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        phone: item.phone.clone(),
        sequence_text: item.sequence.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_rule_views(items: &[RollbackRule]) -> Vec<RollbackRuleView> {
    let mut views: Vec<RollbackRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_rule_view(item));
    }
    views
}

pub fn sum_rollback_rule_view(view: &RollbackRuleView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_rotation_days as f64))
}
