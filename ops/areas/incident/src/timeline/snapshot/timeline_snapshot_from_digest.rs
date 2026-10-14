use super::*;

use std::collections::BTreeMap;

use crate::timeline::digest as timeline_digest;

pub fn summarize_timeline_digests_for_snapshot(items: &[timeline_digest::TimelineDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_timeline_snapshots_from_digest(items: &[timeline_digest::TimelineDigest]) -> Vec<TimelineSnapshot> {
    let mut out: Vec<TimelineSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_snapshot(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_timeline_snapshot(item: &TimelineSnapshot) -> f64 {
    timeline_digest::timeline_digest_weight(6, true)
}
