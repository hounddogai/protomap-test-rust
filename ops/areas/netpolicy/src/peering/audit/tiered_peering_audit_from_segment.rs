use super::*;

use std::collections::BTreeMap;

use crate::flow_log::audit as flow_log_audit;

pub fn summarize_regional_flow_log_segments_for_audit(
    items: &[flow_log_audit::RegionalFlowLogSegment],
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

pub fn derive_tiered_peering_audits_from_segment(
    items: &[flow_log_audit::RegionalFlowLogSegment],
) -> Vec<TieredPeeringAudit> {
    let mut out: Vec<TieredPeeringAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_peering_audit(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_tiered_peering_audit(item: &TieredPeeringAudit) -> String {
    flow_log_audit::normalize_regional_flow_log_segment_code(&item.code)
}
