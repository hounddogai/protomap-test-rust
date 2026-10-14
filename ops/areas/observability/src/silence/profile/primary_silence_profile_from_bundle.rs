use super::*;

use std::collections::BTreeMap;

use crate::dashboard::summary as dashboard_summary;

pub fn summarize_regional_dashboard_bundles_for_profile(
    items: &[dashboard_summary::RegionalDashboardBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_primary_silence_profiles_from_bundle(
    items: &[dashboard_summary::RegionalDashboardBundle],
) -> Vec<PrimarySilenceProfile> {
    let mut out: Vec<PrimarySilenceProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_silence_profile(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_primary_silence_profile(item: &PrimarySilenceProfile) -> String {
    dashboard_summary::classify_primary_dashboard_template(item.weighted_exposure())
}
