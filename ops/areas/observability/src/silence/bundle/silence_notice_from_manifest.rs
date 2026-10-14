use super::*;

use std::collections::BTreeMap;

use crate::silence::reservation as silence_reservation;

pub fn summarize_silence_manifests_for_notice(items: &[silence_reservation::SilenceManifest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_silence_notices_from_manifest(items: &[silence_reservation::SilenceManifest]) -> Vec<SilenceNotice> {
    let mut out: Vec<SilenceNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_notice(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_silence_notice(item: &SilenceNotice) -> String {
    silence_reservation::classify_adaptive_silence_assignment(item.projected_total())
}
