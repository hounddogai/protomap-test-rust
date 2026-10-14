use super::*;

use std::collections::BTreeMap;

use crate::segment::draft as segment_draft;

pub fn summarize_segment_drafts_for_adjustment(items: &[segment_draft::SegmentDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_batched_rule_adjustments_from_draft(items: &[segment_draft::SegmentDraft]) -> Vec<BatchedRuleAdjustment> {
    let mut out: Vec<BatchedRuleAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_rule_adjustment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_batched_rule_adjustment(item: &BatchedRuleAdjustment) -> String {
    segment_draft::normalize_tiered_segment_review_code(&item.code)
}
