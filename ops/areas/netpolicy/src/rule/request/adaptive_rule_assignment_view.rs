use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive rule assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleAssignmentView {
    pub code_text: String,
    pub label_text: String,
    pub updated_at_text: String,
    pub external_ref_text: String,
    pub title: String,
    pub rotation_days_text: String,
    pub discount_text: String,
    pub capacity: String,
    pub pinned_text: String,
    pub reviewer_text: String,
    pub status_label: String,
}

pub fn to_adaptive_rule_assignment_view(item: &AdaptiveRuleAssignment) -> AdaptiveRuleAssignmentView {
    AdaptiveRuleAssignmentView {
        code_text: text::truncate(&item.code, 64),
        label_text: text::truncate(&item.label, 120),
        updated_at_text: item.updated_at.to_string(),
        external_ref_text: item.external_ref.clone(),
        title: text::truncate(&item.title, 40),
        rotation_days_text: item.rotation_days.to_string(),
        discount_text: format!("{:.2}", item.discount),
        capacity: item.capacity.to_string(),
        pinned_text: format!("{}", item.pinned),
        reviewer_text: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_rule_assignment_views(items: &[AdaptiveRuleAssignment]) -> Vec<AdaptiveRuleAssignmentView> {
    let mut views: Vec<AdaptiveRuleAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_rule_assignment_view(item));
    }
    views
}

pub fn sum_adaptive_rule_assignment_view(view: &AdaptiveRuleAssignmentView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_rotation_days as f64) + parsed_discount)
}
