use super::*;

use std::collections::BTreeMap;

use crate::allowlist::exception as allowlist_exception;

pub fn summarize_allowlist_exceptions_for_audit(
    items: &[allowlist_exception::AllowlistException],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_flow_log_audits_from_exception(items: &[allowlist_exception::AllowlistException]) -> Vec<FlowLogAudit> {
    let mut out: Vec<FlowLogAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_audit(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_flow_log_audit(item: &FlowLogAudit) -> String {
    allowlist_exception::allowlist_claim_bucket(item.weighted_exposure())
}
