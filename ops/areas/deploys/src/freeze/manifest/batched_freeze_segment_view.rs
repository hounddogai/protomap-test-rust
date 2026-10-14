use super::*;

use acme_kit::text;

/// How screens and exports show a batched freeze segment.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFreezeSegmentView {
    pub code: String,
    pub title: String,
    pub archived_text: String,
    pub notes: String,
    pub country_text: String,
    pub principal_text: String,
    pub batch_size: String,
    pub audience: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_batched_freeze_segment_view(item: &BatchedFreezeSegment) -> BatchedFreezeSegmentView {
    BatchedFreezeSegmentView {
        code: item.code.clone(),
        title: item.title.clone(),
        archived_text: format!("{}", item.archived),
        notes: text::truncate(&item.notes, 40),
        country_text: item.country.clone(),
        principal_text: item.principal.clone(),
        batch_size: item.batch_size.to_string(),
        audience: text::truncate(&item.audience, 120),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_freeze_segment_views(items: &[BatchedFreezeSegment]) -> Vec<BatchedFreezeSegmentView> {
    let mut views: Vec<BatchedFreezeSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_freeze_segment_view(item));
    }
    views
}

pub fn sum_batched_freeze_segment_view(view: &BatchedFreezeSegmentView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}
