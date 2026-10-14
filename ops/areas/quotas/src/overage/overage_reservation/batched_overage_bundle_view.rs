use super::*;

use acme_kit::text;

/// How screens and exports show a batched overage bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedOverageBundleView {
    pub code: String,
    pub label: String,
    pub expires_at_text: String,
    pub phone_text: String,
    pub channel: String,
    pub margin: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_batched_overage_bundle_view(item: &BatchedOverageBundle) -> BatchedOverageBundleView {
    BatchedOverageBundleView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        expires_at_text: item.expires_at.to_string(),
        phone_text: text::truncate(&item.phone, 80),
        channel: item.channel.clone(),
        margin: format!("{:.2}", item.margin),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_overage_bundle_views(items: &[BatchedOverageBundle]) -> Vec<BatchedOverageBundleView> {
    let mut views: Vec<BatchedOverageBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_overage_bundle_view(item));
    }
    views
}

pub fn sum_batched_overage_bundle_view(view: &BatchedOverageBundleView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_margin)
}
