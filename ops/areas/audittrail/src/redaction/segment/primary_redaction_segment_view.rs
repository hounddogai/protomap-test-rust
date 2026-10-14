use super::*;

use acme_kit::text;

/// How screens and exports show a primary redaction segment.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionSegmentView {
    pub code_text: String,
    pub title: String,
    pub ceiling: String,
    pub retries_text: String,
    pub namespace: String,
    pub unit: String,
    pub expires_at: String,
    pub weight_text: String,
    pub status_label: String,
}

pub fn to_primary_redaction_segment_view(item: &PrimaryRedactionSegment) -> PrimaryRedactionSegmentView {
    PrimaryRedactionSegmentView {
        code_text: text::truncate(&item.code, 120),
        title: item.title.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        retries_text: item.retries.to_string(),
        namespace: item.namespace.clone(),
        unit: item.unit.clone(),
        expires_at: item.expires_at.to_string(),
        weight_text: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_redaction_segment_views(items: &[PrimaryRedactionSegment]) -> Vec<PrimaryRedactionSegmentView> {
    let mut views: Vec<PrimaryRedactionSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_redaction_segment_view(item));
    }
    views
}

pub fn sum_primary_redaction_segment_view(view: &PrimaryRedactionSegmentView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_ceiling + (parsed_retries as f64) + (parsed_expires_at as f64))
}
