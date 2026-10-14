use super::*;

use acme_kit::text;

/// How screens and exports show a alert bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertBundleView {
    pub code: String,
    pub title: String,
    pub principal_text: String,
    pub channel: String,
    pub threshold: String,
    pub scope: String,
    pub ends_at: String,
    pub label: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_alert_bundle_view(item: &AlertBundle) -> AlertBundleView {
    AlertBundleView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        principal_text: text::truncate(&item.principal, 80),
        channel: item.channel.clone(),
        threshold: format!("{:.2}", item.threshold),
        scope: item.scope.clone(),
        ends_at: item.ends_at.to_string(),
        label: item.label.clone(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_bundle_views(items: &[AlertBundle]) -> Vec<AlertBundleView> {
    let mut views: Vec<AlertBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_bundle_view(item));
    }
    views
}

pub fn sum_alert_bundle_view(view: &AlertBundleView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_threshold + (parsed_ends_at as f64))
}
