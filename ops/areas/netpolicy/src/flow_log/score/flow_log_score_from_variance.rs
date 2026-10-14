use super::*;

use std::collections::BTreeMap;

use crate::flow_log::manifest as flow_log_manifest;

pub fn summarize_flow_log_variances_for_score(items: &[flow_log_manifest::FlowLogVariance]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_flow_log_scores_from_variance(items: &[flow_log_manifest::FlowLogVariance]) -> Vec<FlowLogScore> {
    let mut out: Vec<FlowLogScore> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_score(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_flow_log_score(item: &FlowLogScore) -> String {
    flow_log_manifest::classify_flow_log_manifest(item.projected_volume())
}
