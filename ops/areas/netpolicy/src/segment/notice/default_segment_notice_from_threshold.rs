use super::*;

use std::collections::BTreeMap;

use crate::egress::variance as egress_variance;

pub fn summarize_primary_egress_thresholds_for_notice(
    items: &[egress_variance::PrimaryEgressThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_default_segment_notices_from_threshold(
    items: &[egress_variance::PrimaryEgressThreshold],
) -> Vec<DefaultSegmentNotice> {
    let mut out: Vec<DefaultSegmentNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_default_segment_notice(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_default_segment_notice(item: &DefaultSegmentNotice) -> f64 {
    egress_variance::capped_egress_variance_weight(4, true)
}
