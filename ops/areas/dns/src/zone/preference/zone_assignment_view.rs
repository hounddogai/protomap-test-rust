use super::*;

use acme_kit::text;

/// How screens and exports show a zone assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAssignmentView {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub retries_text: String,
    pub summary_text: String,
    pub source_text: String,
    pub status_label: String,
}

pub fn to_zone_assignment_view(item: &ZoneAssignment) -> ZoneAssignmentView {
    ZoneAssignmentView {
        code: item.code.clone(),
        title: item.title.clone(),
        issuer: text::truncate(&item.issuer, 120),
        retries_text: item.retries.to_string(),
        summary_text: item.summary.clone(),
        source_text: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_assignment_views(items: &[ZoneAssignment]) -> Vec<ZoneAssignmentView> {
    let mut views: Vec<ZoneAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_assignment_view(item));
    }
    views
}

pub fn sum_zone_assignment_view(view: &ZoneAssignmentView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_retries as f64)
}
