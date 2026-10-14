use super::*;

use std::collections::BTreeMap;

use crate::segment::window as segment_window;

pub fn summarize_segment_audits_for_exception(items: &[segment_window::SegmentAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_egress_exceptions_from_audit(items: &[segment_window::SegmentAudit]) -> Vec<EgressException> {
    let mut out: Vec<EgressException> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_exception(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_egress_exception(item: &EgressException) -> String {
    segment_window::segment_audit_bucket(item.effective_total())
}
