use super::*;

use std::collections::BTreeMap;

use crate::grant::record_item as grant_record_item;

pub fn summarize_lenient_grant_records_for_digest(
    items: &[grant_record_item::LenientGrantRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_overage_digests_from_record(items: &[grant_record_item::LenientGrantRecord]) -> Vec<OverageDigest> {
    let mut out: Vec<OverageDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_digest(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_overage_digest(item: &OverageDigest) -> String {
    grant_record_item::normalize_grant_snapshot_code(&item.code)
}
