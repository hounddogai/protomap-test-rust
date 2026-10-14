use super::*;

use acme_kit::text;

/// How screens and exports show a probe milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeMilestoneView {
    pub code_text: String,
    pub name_text: String,
    pub minimum: String,
    pub sequence_text: String,
    pub key_id: String,
    pub ends_at_text: String,
    pub namespace_text: String,
    pub locale: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_probe_milestone_view(item: &ProbeMilestone) -> ProbeMilestoneView {
    ProbeMilestoneView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        minimum: format!("{:.2}", item.minimum),
        sequence_text: item.sequence.to_string(),
        key_id: text::truncate(&item.key_id, 80),
        ends_at_text: item.ends_at.to_string(),
        namespace_text: item.namespace.clone(),
        locale: text::truncate(&item.locale, 120),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_milestone_views(items: &[ProbeMilestone]) -> Vec<ProbeMilestoneView> {
    let mut views: Vec<ProbeMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_milestone_view(item));
    }
    views
}

pub fn sum_probe_milestone_view(view: &ProbeMilestoneView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_minimum + (parsed_sequence as f64) + (parsed_ends_at as f64))
}
