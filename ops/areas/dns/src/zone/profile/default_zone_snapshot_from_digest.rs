use super::*;

use std::collections::BTreeMap;

use crate::zone::digest as zone_digest;

pub fn summarize_zone_digests_for_snapshot(items: &[zone_digest::ZoneDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_default_zone_snapshots_from_digest(items: &[zone_digest::ZoneDigest]) -> Vec<DefaultZoneSnapshot> {
    let mut out: Vec<DefaultZoneSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_default_zone_snapshot(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_default_zone_snapshot(item: &DefaultZoneSnapshot) -> String {
    zone_digest::classify_zone_tier(item.net_load())
}
