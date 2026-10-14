use super::*;

use std::collections::BTreeMap;

use crate::egress::cycle as egress_cycle;

pub fn summarize_egress_notices_for_proposal(items: &[egress_cycle::EgressNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_flow_log_proposals_from_notice(items: &[egress_cycle::EgressNotice]) -> Vec<FlowLogProposal> {
    let mut out: Vec<FlowLogProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_proposal(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_flow_log_proposal(item: &FlowLogProposal) -> String {
    egress_cycle::classify_egress_notice(item.measured_value())
}
