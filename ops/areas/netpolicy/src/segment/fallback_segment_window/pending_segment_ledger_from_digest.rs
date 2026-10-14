use super::*;

use std::collections::BTreeMap;

use crate::egress::digest as egress_digest;

pub fn summarize_egress_digests_for_ledger(items: &[egress_digest::EgressDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_pending_segment_ledgers_from_digest(items: &[egress_digest::EgressDigest]) -> Vec<PendingSegmentLedger> {
    let mut out: Vec<PendingSegmentLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_segment_ledger(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_pending_segment_ledger(item: &PendingSegmentLedger) -> f64 {
    egress_digest::tiered_egress_digest_weight(6, true)
}
