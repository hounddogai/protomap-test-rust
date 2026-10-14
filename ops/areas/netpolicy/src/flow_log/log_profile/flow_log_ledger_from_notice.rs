use super::*;

use std::collections::BTreeMap;

use crate::flow_log::draft as flow_log_draft;

pub fn summarize_pending_flow_log_notices_for_ledger(
    items: &[flow_log_draft::PendingFlowLogNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_flow_log_ledgers_from_notice(items: &[flow_log_draft::PendingFlowLogNotice]) -> Vec<FlowLogLedger> {
    let mut out: Vec<FlowLogLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_ledger(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_flow_log_ledger_extra(item: &FlowLogLedger) -> String {
    flow_log_draft::normalize_tiered_flow_log_manifest_code(&item.code)
}
