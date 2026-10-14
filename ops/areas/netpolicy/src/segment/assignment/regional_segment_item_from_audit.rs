use super::*;

use std::collections::BTreeMap;

use crate::flow_log::claim as flow_log_claim;

pub fn summarize_strict_flow_log_audits_for_item(
    items: &[flow_log_claim::StrictFlowLogAudit],
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

pub fn derive_regional_segment_items_from_audit(
    items: &[flow_log_claim::StrictFlowLogAudit],
) -> Vec<RegionalSegmentItem> {
    let mut out: Vec<RegionalSegmentItem> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_segment_item(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_regional_segment_item(item: &RegionalSegmentItem) -> String {
    flow_log_claim::normalize_flow_log_claim_code(&item.code)
}
