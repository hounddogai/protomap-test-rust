use super::*;

use std::collections::BTreeMap;

use crate::silence::silence_metric as silence_silence_metric;

pub fn summarize_strict_silence_preferences_for_digest(
    items: &[silence_silence_metric::StrictSilencePreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_strict_dashboard_digests_from_preference(
    items: &[silence_silence_metric::StrictSilencePreference],
) -> Vec<StrictDashboardDigest> {
    let mut out: Vec<StrictDashboardDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_dashboard_digest(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_strict_dashboard_digest(item: &StrictDashboardDigest) -> String {
    silence_silence_metric::normalize_strict_silence_preference_code(&item.code)
}
