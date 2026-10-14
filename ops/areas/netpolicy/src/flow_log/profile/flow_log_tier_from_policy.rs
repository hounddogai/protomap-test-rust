use super::*;

use std::collections::BTreeMap;

use crate::segment::policy as segment_policy;

pub fn summarize_segment_policies_for_tier(items: &[segment_policy::SegmentPolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_flow_log_tiers_from_policy(items: &[segment_policy::SegmentPolicy]) -> Vec<FlowLogTier> {
    let mut out: Vec<FlowLogTier> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_tier(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_flow_log_tier(item: &FlowLogTier) -> String {
    segment_policy::normalize_segment_estimate_code(&item.code)
}
