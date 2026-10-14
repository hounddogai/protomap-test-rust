use super::*;

use std::collections::BTreeMap;

use crate::flow_log::estimate as flow_log_estimate;

pub fn summarize_pending_flow_log_estimates_for_record(
    items: &[flow_log_estimate::PendingFlowLogEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_lenient_egress_records_from_estimate(
    items: &[flow_log_estimate::PendingFlowLogEstimate],
) -> Vec<LenientEgressRecord> {
    let mut out: Vec<LenientEgressRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_egress_record(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_lenient_egress_record(item: &LenientEgressRecord) -> String {
    flow_log_estimate::normalize_tiered_flow_log_window_code(&item.code)
}
