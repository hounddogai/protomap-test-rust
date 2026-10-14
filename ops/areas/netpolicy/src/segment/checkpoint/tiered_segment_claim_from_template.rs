use super::*;

use std::collections::BTreeMap;

use crate::segment::template as segment_template;

pub fn summarize_segment_templates_for_claim(items: &[segment_template::SegmentTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_tiered_segment_claims_from_template(
    items: &[segment_template::SegmentTemplate],
) -> Vec<TieredSegmentClaim> {
    let mut out: Vec<TieredSegmentClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_segment_claim(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_tiered_segment_claim(item: &TieredSegmentClaim) -> String {
    segment_template::normalize_segment_snapshot_code(&item.code)
}
