use super::*;

use std::collections::BTreeMap;

use crate::egress::manifest as egress_manifest;

pub fn summarize_fallback_egress_thresholds_for_segment(
    items: &[egress_manifest::FallbackEgressThreshold],
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

pub fn derive_peering_segments_from_threshold(
    items: &[egress_manifest::FallbackEgressThreshold],
) -> Vec<PeeringSegment> {
    let mut out: Vec<PeeringSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_segment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_peering_segment(item: &PeeringSegment) -> f64 {
    egress_manifest::fallback_egress_threshold_weight(2, true)
}
