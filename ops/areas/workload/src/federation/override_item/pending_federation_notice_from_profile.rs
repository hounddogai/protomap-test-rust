use super::*;

use std::collections::BTreeMap;

use crate::federation::schedule as federation_schedule;

pub fn summarize_strict_federation_profiles_for_notice(
    items: &[federation_schedule::StrictFederationProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_pending_federation_notices_from_profile(
    items: &[federation_schedule::StrictFederationProfile],
) -> Vec<PendingFederationNotice> {
    let mut out: Vec<PendingFederationNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_federation_notice(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_pending_federation_notice(item: &PendingFederationNotice) -> String {
    federation_schedule::strict_federation_profile_bucket(item.effective_load())
}
