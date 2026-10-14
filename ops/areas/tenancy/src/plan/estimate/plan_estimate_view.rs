use super::*;

use acme_kit::text;

/// How screens and exports show a plan estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanEstimateView {
    pub code_text: String,
    pub name_text: String,
    pub batch_size_text: String,
    pub revision_text: String,
    pub threshold: String,
    pub host: String,
    pub ttl_seconds_text: String,
    pub created_at_text: String,
    pub contact_email: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_plan_estimate_view(item: &PlanEstimate) -> PlanEstimateView {
    PlanEstimateView {
        code_text: text::truncate(&item.code, 40),
        name_text: text::truncate(&item.name, 80),
        batch_size_text: item.batch_size.to_string(),
        revision_text: item.revision.to_string(),
        threshold: format!("{:.2}", item.threshold),
        host: item.host.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        created_at_text: item.created_at.to_string(),
        contact_email: text::truncate(&item.contact_email, 40),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_estimate_views(items: &[PlanEstimate]) -> Vec<PlanEstimateView> {
    let mut views: Vec<PlanEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_estimate_view(item));
    }
    views
}

pub fn sum_plan_estimate_view(view: &PlanEstimateView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_revision as f64) + parsed_threshold)
}
