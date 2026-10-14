use super::*;

use std::collections::BTreeMap;

use crate::audit_event::event_rollup as audit_event_event_rollup;

pub fn summarize_global_audit_event_notices_for_ledger(
    items: &[audit_event_event_rollup::GlobalAuditEventNotice],
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

pub fn derive_legacy_sink_ledgers_from_notice(
    items: &[audit_event_event_rollup::GlobalAuditEventNotice],
) -> Vec<LegacySinkLedger> {
    let mut out: Vec<LegacySinkLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_sink_ledger(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_legacy_sink_ledger(item: &LegacySinkLedger) -> String {
    audit_event_event_rollup::normalize_global_audit_event_notice_code(&item.code)
}
