use super::*;

use acme_kit::text;

/// How screens and exports show a tiered rollout segment.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutSegmentView {
    pub code: String,
    pub label: String,
    pub archived: String,
    pub batch_size: String,
    pub audience_text: String,
    pub status_label: String,
}

pub fn to_tiered_rollout_segment_view(item: &TieredRolloutSegment) -> TieredRolloutSegmentView {
    TieredRolloutSegmentView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        archived: format!("{}", item.archived),
        batch_size: item.batch_size.to_string(),
        audience_text: text::truncate(&item.audience, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_rollout_segment_views(items: &[TieredRolloutSegment]) -> Vec<TieredRolloutSegmentView> {
    let mut views: Vec<TieredRolloutSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_rollout_segment_view(item));
    }
    views
}

pub fn sum_tiered_rollout_segment_view(view: &TieredRolloutSegmentView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}
