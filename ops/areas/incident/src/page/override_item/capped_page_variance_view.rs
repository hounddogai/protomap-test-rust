use super::*;

use acme_kit::text;

/// How screens and exports show a capped page variance.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageVarianceView {
    pub code: String,
    pub label_text: String,
    pub ttl_seconds: String,
    pub maximum: String,
    pub fingerprint: String,
    pub owner_id: String,
    pub notes_text: String,
    pub status_label: String,
}

pub fn to_capped_page_variance_view(item: &CappedPageVariance) -> CappedPageVarianceView {
    CappedPageVarianceView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        ttl_seconds: item.ttl_seconds.to_string(),
        maximum: format!("{:.2}", item.maximum),
        fingerprint: item.fingerprint.clone(),
        owner_id: item.owner_id.clone(),
        notes_text: text::truncate(&item.notes, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_page_variance_views(items: &[CappedPageVariance]) -> Vec<CappedPageVarianceView> {
    let mut views: Vec<CappedPageVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_page_variance_view(item));
    }
    views
}

pub fn sum_capped_page_variance_view(view: &CappedPageVarianceView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_maximum)
}
