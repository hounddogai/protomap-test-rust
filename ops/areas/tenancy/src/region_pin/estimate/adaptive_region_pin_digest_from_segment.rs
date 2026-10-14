use super::*;

use std::collections::BTreeMap;

use crate::tenant::segment as tenant_segment;

pub fn summarize_tenant_segments_for_digest(items: &[tenant_segment::TenantSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_adaptive_region_pin_digests_from_segment(
    items: &[tenant_segment::TenantSegment],
) -> Vec<AdaptiveRegionPinDigest> {
    let mut out: Vec<AdaptiveRegionPinDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_region_pin_digest(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_adaptive_region_pin_digest_extra(item: &AdaptiveRegionPinDigest) -> f64 {
    tenant_segment::manual_tenant_notice_weight(7, true)
}
