use super::*;

use std::collections::BTreeMap;

use crate::silence::silence_variance as silence_silence_variance;

pub fn summarize_weighted_silence_entries_for_window(
    items: &[silence_silence_variance::WeightedSilenceEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_global_dashboard_windows_from_entry(
    items: &[silence_silence_variance::WeightedSilenceEntry],
) -> Vec<GlobalDashboardWindow> {
    let mut out: Vec<GlobalDashboardWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_global_dashboard_window(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_global_dashboard_window(item: &GlobalDashboardWindow) -> String {
    silence_silence_variance::regional_silence_variance_bucket(item.weighted_load())
}
