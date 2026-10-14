use super::*;

use std::collections::BTreeMap;

use crate::segment::window as segment_window;

pub fn summarize_segment_audits_for_draft(items: &[segment_window::SegmentAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_flow_log_drafts_from_audit(items: &[segment_window::SegmentAudit]) -> Vec<FlowLogDraft> {
    let mut out: Vec<FlowLogDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_draft(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_flow_log_draft(item: &FlowLogDraft) -> f64 {
    segment_window::strict_segment_window_weight(3, true)
}
