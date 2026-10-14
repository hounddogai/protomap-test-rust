use super::*;

/// How screens and exports show a scheduled rule milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleMilestoneView {
    pub code: String,
    pub label: String,
    pub starts_at_text: String,
    pub locale_text: String,
    pub status_label: String,
}

pub fn to_scheduled_rule_milestone_view(item: &ScheduledRuleMilestone) -> ScheduledRuleMilestoneView {
    ScheduledRuleMilestoneView {
        code: item.code.clone(),
        label: item.label.clone(),
        starts_at_text: item.starts_at.to_string(),
        locale_text: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_rule_milestone_views(items: &[ScheduledRuleMilestone]) -> Vec<ScheduledRuleMilestoneView> {
    let mut views: Vec<ScheduledRuleMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_rule_milestone_view(item));
    }
    views
}

pub fn sum_scheduled_rule_milestone_view(view: &ScheduledRuleMilestoneView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_starts_at as f64)
}
