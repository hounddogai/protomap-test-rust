use super::*;

use std::collections::BTreeMap;

use crate::segment::proposal as segment_proposal;

pub fn summarize_weighted_segment_forecasts_for_draft(
    items: &[segment_proposal::WeightedSegmentForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_fallback_segment_drafts_from_forecast(
    items: &[segment_proposal::WeightedSegmentForecast],
) -> Vec<FallbackSegmentDraft> {
    let mut out: Vec<FallbackSegmentDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_segment_draft(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_fallback_segment_draft(item: &FallbackSegmentDraft) -> String {
    segment_proposal::normalize_fallback_segment_proposal_code(&item.code)
}
