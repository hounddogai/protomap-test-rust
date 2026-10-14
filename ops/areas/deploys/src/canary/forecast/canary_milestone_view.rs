use super::*;

use acme_kit::text;

/// How screens and exports show a canary milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryMilestoneView {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub maximum_text: String,
    pub limit_text: String,
    pub burst: String,
    pub summary: String,
    pub minimum_text: String,
    pub margin: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_canary_milestone_view(item: &CanaryMilestone) -> CanaryMilestoneView {
    CanaryMilestoneView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        fingerprint: text::truncate(&item.fingerprint, 80),
        maximum_text: format!("{:.2}", item.maximum),
        limit_text: item.limit.to_string(),
        burst: item.burst.to_string(),
        summary: item.summary.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        margin: format!("{:.2}", item.margin),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_milestone_views(items: &[CanaryMilestone]) -> Vec<CanaryMilestoneView> {
    let mut views: Vec<CanaryMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_milestone_view(item));
    }
    views
}

pub fn sum_canary_milestone_view(view: &CanaryMilestoneView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_maximum + (parsed_limit as f64) + (parsed_burst as f64))
}
