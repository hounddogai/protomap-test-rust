use super::*;

use std::collections::BTreeMap;

use crate::token::snapshot as token_snapshot;

pub fn summarize_token_snapshots_for_segment(items: &[token_snapshot::TokenSnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_federation_segments_from_snapshot(items: &[token_snapshot::TokenSnapshot]) -> Vec<FederationSegment> {
    let mut out: Vec<FederationSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_federation_segment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_federation_segment(item: &FederationSegment) -> String {
    token_snapshot::tiered_token_bundle_bucket(item.projected_load())
}
