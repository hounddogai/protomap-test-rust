use super::*;

use std::collections::BTreeMap;

use crate::segment::draft as segment_draft;

pub fn summarize_tiered_segment_reviews_for_audit(
    items: &[segment_draft::TieredSegmentReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_strict_flow_log_audits_from_review(
    items: &[segment_draft::TieredSegmentReview],
) -> Vec<StrictFlowLogAudit> {
    let mut out: Vec<StrictFlowLogAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_flow_log_audit(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_strict_flow_log_audit(item: &StrictFlowLogAudit) -> f64 {
    segment_draft::strict_segment_ledger_weight(5, true)
}
