use super::*;

use std::collections::BTreeMap;

use crate::egress::agreement as egress_agreement;

pub fn summarize_egress_rollups_for_segment(items: &[egress_agreement::EgressRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_allowlist_segments_from_rollup(items: &[egress_agreement::EgressRollup]) -> Vec<AllowlistSegment> {
    let mut out: Vec<AllowlistSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_segment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_allowlist_segment(item: &AllowlistSegment) -> String {
    egress_agreement::classify_egress_preference(item.adjusted_exposure())
}
