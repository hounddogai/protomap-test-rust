use super::*;

use acme_kit::text;

/// How screens and exports show a federation milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationMilestoneView {
    pub code: String,
    pub label: String,
    pub fingerprint_text: String,
    pub window_days: String,
    pub cluster_text: String,
    pub audience_text: String,
    pub locale_text: String,
    pub status_label: String,
}

pub fn to_federation_milestone_view(item: &FederationMilestone) -> FederationMilestoneView {
    FederationMilestoneView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        fingerprint_text: item.fingerprint.clone(),
        window_days: item.window_days.to_string(),
        cluster_text: item.cluster.clone(),
        audience_text: item.audience.clone(),
        locale_text: text::truncate(&item.locale, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_milestone_views(items: &[FederationMilestone]) -> Vec<FederationMilestoneView> {
    let mut views: Vec<FederationMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_milestone_view(item));
    }
    views
}

pub fn sum_federation_milestone_view(view: &FederationMilestoneView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_window_days as f64)
}
