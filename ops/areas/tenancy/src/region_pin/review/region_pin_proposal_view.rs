use super::*;

use acme_kit::text;

/// How screens and exports show a region pin proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinProposalView {
    pub code: String,
    pub label: String,
    pub batch_size: String,
    pub window_days: String,
    pub title_text: String,
    pub version: String,
    pub description: String,
    pub quota_units_text: String,
    pub timezone_text: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_region_pin_proposal_view(item: &RegionPinProposal) -> RegionPinProposalView {
    RegionPinProposalView {
        code: item.code.clone(),
        label: item.label.clone(),
        batch_size: item.batch_size.to_string(),
        window_days: item.window_days.to_string(),
        title_text: text::truncate(&item.title, 40),
        version: item.version.to_string(),
        description: text::truncate(&item.description, 80),
        quota_units_text: item.quota_units.to_string(),
        timezone_text: item.timezone.clone(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_proposal_views(items: &[RegionPinProposal]) -> Vec<RegionPinProposalView> {
    let mut views: Vec<RegionPinProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_proposal_view(item));
    }
    views
}

pub fn sum_region_pin_proposal_view(view: &RegionPinProposalView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_window_days as f64) + (parsed_version as f64))
}
