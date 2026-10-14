use super::*;

use std::collections::BTreeMap;

use crate::redaction::manifest as redaction_manifest;

pub fn summarize_lenient_redaction_manifests_for_allocation(
    items: &[redaction_manifest::LenientRedactionManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_sink_allocations_from_manifest(
    items: &[redaction_manifest::LenientRedactionManifest],
) -> Vec<SinkAllocation> {
    let mut out: Vec<SinkAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_allocation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_sink_allocation(item: &SinkAllocation) -> f64 {
    redaction_manifest::lenient_redaction_manifest_weight(4, true)
}
