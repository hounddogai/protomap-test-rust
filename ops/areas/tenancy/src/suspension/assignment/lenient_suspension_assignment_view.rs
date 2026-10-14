use super::*;

use acme_kit::text;

/// How screens and exports show a lenient suspension assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionAssignmentView {
    pub code_text: String,
    pub title: String,
    pub starts_at: String,
    pub ceiling_text: String,
    pub reviewer: String,
    pub unit_text: String,
    pub channel: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_lenient_suspension_assignment_view(item: &LenientSuspensionAssignment) -> LenientSuspensionAssignmentView {
    LenientSuspensionAssignmentView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        starts_at: item.starts_at.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        reviewer: item.reviewer.clone(),
        unit_text: item.unit.clone(),
        channel: text::truncate(&item.channel, 80),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_suspension_assignment_views(
    items: &[LenientSuspensionAssignment],
) -> Vec<LenientSuspensionAssignmentView> {
    let mut views: Vec<LenientSuspensionAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_suspension_assignment_view(item));
    }
    views
}

pub fn sum_lenient_suspension_assignment_view(view: &LenientSuspensionAssignmentView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_ceiling)
}
