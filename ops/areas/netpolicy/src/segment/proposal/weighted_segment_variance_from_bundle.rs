use super::*;

use std::collections::BTreeMap;

use crate::egress::group as egress_group;

pub fn summarize_manual_egress_bundles_for_variance(
    items: &[egress_group::ManualEgressBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_weighted_segment_variances_from_bundle(
    items: &[egress_group::ManualEgressBundle],
) -> Vec<WeightedSegmentVariance> {
    let mut out: Vec<WeightedSegmentVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_segment_variance(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_weighted_segment_variance(item: &WeightedSegmentVariance) -> String {
    egress_group::tiered_egress_cycle_bucket(item.weighted_value())
}
