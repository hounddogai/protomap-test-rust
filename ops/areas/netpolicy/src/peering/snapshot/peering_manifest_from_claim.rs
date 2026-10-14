use super::*;

use std::collections::BTreeMap;

use crate::segment::checkpoint as segment_checkpoint;

pub fn summarize_tiered_segment_claims_for_manifest(
    items: &[segment_checkpoint::TieredSegmentClaim],
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

pub fn derive_peering_manifests_from_claim(items: &[segment_checkpoint::TieredSegmentClaim]) -> Vec<PeeringManifest> {
    let mut out: Vec<PeeringManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_manifest(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_peering_manifest(item: &PeeringManifest) -> String {
    segment_checkpoint::classify_segment_preference(item.effective_value())
}
