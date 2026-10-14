use super::*;

use acme_kit::text;

/// How screens and exports show a lenient data export milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDataExportMilestoneView {
    pub code_text: String,
    pub title: String,
    pub created_at_text: String,
    pub external_ref: String,
    pub pinned_text: String,
    pub status_label: String,
}

pub fn to_lenient_data_export_milestone_view(item: &LenientDataExportMilestone) -> LenientDataExportMilestoneView {
    LenientDataExportMilestoneView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        created_at_text: item.created_at.to_string(),
        external_ref: text::truncate(&item.external_ref, 120),
        pinned_text: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_data_export_milestone_views(
    items: &[LenientDataExportMilestone],
) -> Vec<LenientDataExportMilestoneView> {
    let mut views: Vec<LenientDataExportMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_data_export_milestone_view(item));
    }
    views
}

pub fn sum_lenient_data_export_milestone_view(view: &LenientDataExportMilestoneView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_created_at as f64)
}
