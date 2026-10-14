use super::*;

use acme_kit::text;

/// How screens and exports show a capped token segment.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenSegmentView {
    pub code: String,
    pub title_text: String,
    pub audience_text: String,
    pub issuer: String,
    pub principal_text: String,
    pub quota_units: String,
    pub segment: String,
    pub ratio_text: String,
    pub priority: String,
    pub country: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_capped_token_segment_view(item: &CappedTokenSegment) -> CappedTokenSegmentView {
    CappedTokenSegmentView {
        code: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        audience_text: text::truncate(&item.audience, 120),
        issuer: item.issuer.clone(),
        principal_text: item.principal.clone(),
        quota_units: item.quota_units.to_string(),
        segment: text::truncate(&item.segment, 120),
        ratio_text: format!("{:.2}", item.ratio),
        priority: item.priority.to_string(),
        country: item.country.clone(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_token_segment_views(items: &[CappedTokenSegment]) -> Vec<CappedTokenSegmentView> {
    let mut views: Vec<CappedTokenSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_token_segment_view(item));
    }
    views
}

pub fn sum_capped_token_segment_view(view: &CappedTokenSegmentView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_ratio + (parsed_priority as f64))
}
