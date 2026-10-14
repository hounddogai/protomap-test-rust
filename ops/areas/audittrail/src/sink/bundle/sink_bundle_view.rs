use super::*;

use acme_kit::text;

/// How screens and exports show a sink bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkBundleView {
    pub code: String,
    pub name: String,
    pub budget_text: String,
    pub country: String,
    pub priority: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_sink_bundle_view(item: &SinkBundle) -> SinkBundleView {
    SinkBundleView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        budget_text: format!("{:.2}", item.budget),
        country: item.country.clone(),
        priority: item.priority.to_string(),
        summary: text::truncate(&item.summary, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_bundle_views(items: &[SinkBundle]) -> Vec<SinkBundleView> {
    let mut views: Vec<SinkBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_bundle_view(item));
    }
    views
}

pub fn sum_sink_bundle_view(view: &SinkBundleView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_budget + (parsed_priority as f64))
}
