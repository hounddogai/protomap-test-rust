use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBaselineView {
    pub code: String,
    pub label: String,
    pub threshold: String,
    pub scope: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_postmortem_baseline_view(item: &PostmortemBaseline) -> PostmortemBaselineView {
    PostmortemBaselineView {
        code: item.code.clone(),
        label: item.label.clone(),
        threshold: format!("{:.2}", item.threshold),
        scope: text::truncate(&item.scope, 80),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_baseline_views(items: &[PostmortemBaseline]) -> Vec<PostmortemBaselineView> {
    let mut views: Vec<PostmortemBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_baseline_view(item));
    }
    views
}

pub fn sum_postmortem_baseline_view(view: &PostmortemBaselineView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_threshold + (parsed_batch_size as f64))
}
