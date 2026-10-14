use super::*;

use std::collections::BTreeMap;

use crate::grant::record_item as grant_record_item;

pub fn summarize_grant_snapshots_for_target(items: &[grant_record_item::GrantSnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_grant_targets_from_snapshot(items: &[grant_record_item::GrantSnapshot]) -> Vec<GrantTarget> {
    let mut out: Vec<GrantTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_target(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_grant_target(item: &GrantTarget) -> String {
    grant_record_item::classify_grant_snapshot(item.projected_volume())
}
