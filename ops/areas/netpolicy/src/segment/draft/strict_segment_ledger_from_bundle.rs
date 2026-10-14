use super::*;

use std::collections::BTreeMap;

use crate::allowlist::bundle as allowlist_bundle;

pub fn summarize_global_allowlist_bundles_for_ledger(
    items: &[allowlist_bundle::GlobalAllowlistBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_strict_segment_ledgers_from_bundle(
    items: &[allowlist_bundle::GlobalAllowlistBundle],
) -> Vec<StrictSegmentLedger> {
    let mut out: Vec<StrictSegmentLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_segment_ledger(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_strict_segment_ledger(item: &StrictSegmentLedger) -> String {
    allowlist_bundle::strict_allowlist_bundle_bucket(item.net_total())
}
