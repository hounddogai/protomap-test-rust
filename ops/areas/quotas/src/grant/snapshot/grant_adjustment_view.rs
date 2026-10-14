use super::*;

use acme_kit::text;

/// How screens and exports show a grant adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantAdjustmentView {
    pub code: String,
    pub title: String,
    pub threshold: String,
    pub key_id: String,
    pub retries_text: String,
    pub rate_text: String,
    pub checksum: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_grant_adjustment_view(item: &GrantAdjustment) -> GrantAdjustmentView {
    GrantAdjustmentView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        threshold: format!("{:.2}", item.threshold),
        key_id: item.key_id.clone(),
        retries_text: item.retries.to_string(),
        rate_text: format!("{:.2}", item.rate),
        checksum: item.checksum.clone(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_adjustment_views(items: &[GrantAdjustment]) -> Vec<GrantAdjustmentView> {
    let mut views: Vec<GrantAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_adjustment_view(item));
    }
    views
}

pub fn sum_grant_adjustment_view(view: &GrantAdjustmentView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_threshold + (parsed_retries as f64) + parsed_rate)
}
