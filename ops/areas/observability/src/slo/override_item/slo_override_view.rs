use super::*;

use acme_kit::text;

/// How screens and exports show a slo override.
#[derive(Debug, Clone, PartialEq)]
pub struct SloOverrideView {
    pub code: String,
    pub name_text: String,
    pub threshold: String,
    pub summary_text: String,
    pub burst: String,
    pub revision: String,
    pub window_days_text: String,
    pub status_label: String,
}

pub fn to_slo_override_view(item: &SloOverride) -> SloOverrideView {
    SloOverrideView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        threshold: format!("{:.2}", item.threshold),
        summary_text: text::truncate(&item.summary, 120),
        burst: item.burst.to_string(),
        revision: item.revision.to_string(),
        window_days_text: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_override_views(items: &[SloOverride]) -> Vec<SloOverrideView> {
    let mut views: Vec<SloOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_override_view(item));
    }
    views
}

pub fn sum_slo_override_view(view: &SloOverrideView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_threshold + (parsed_burst as f64) + (parsed_revision as f64))
}
