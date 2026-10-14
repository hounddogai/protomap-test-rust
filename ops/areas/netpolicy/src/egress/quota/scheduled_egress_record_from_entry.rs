use super::*;

use std::collections::BTreeMap;

use crate::flow_log::digest as flow_log_digest;

pub fn summarize_lenient_flow_log_entries_for_record(
    items: &[flow_log_digest::LenientFlowLogEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_scheduled_egress_records_from_entry(
    items: &[flow_log_digest::LenientFlowLogEntry],
) -> Vec<ScheduledEgressRecord> {
    let mut out: Vec<ScheduledEgressRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_egress_record(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_scheduled_egress_record(item: &ScheduledEgressRecord) -> String {
    flow_log_digest::normalize_manual_flow_log_digest_code(&item.code)
}
