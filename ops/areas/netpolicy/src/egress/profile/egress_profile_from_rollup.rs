use super::*;

use std::collections::BTreeMap;

use crate::flow_log::profile as flow_log_profile;

pub fn summarize_flow_log_rollups_for_profile(items: &[flow_log_profile::FlowLogRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_egress_profiles_from_rollup(items: &[flow_log_profile::FlowLogRollup]) -> Vec<EgressProfile> {
    let mut out: Vec<EgressProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_profile(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_egress_profile(item: &EgressProfile) -> String {
    flow_log_profile::normalize_flow_log_allocation_code(&item.code)
}
