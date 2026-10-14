use super::*;

use std::collections::BTreeMap;

use crate::segment::segment_window as segment_segment_window;

pub fn summarize_batched_segment_windows_for_agreement(
    items: &[segment_segment_window::BatchedSegmentWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_capped_flow_log_agreements_from_window(
    items: &[segment_segment_window::BatchedSegmentWindow],
) -> Vec<CappedFlowLogAgreement> {
    let mut out: Vec<CappedFlowLogAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_flow_log_agreement(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_capped_flow_log_agreement(item: &CappedFlowLogAgreement) -> f64 {
    segment_segment_window::default_segment_checkpoint_weight(1, true)
}
