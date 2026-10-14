use super::*;

use std::collections::BTreeMap;

use crate::silence::profile as silence_profile;

pub fn summarize_primary_silence_profiles_for_variance(
    items: &[silence_profile::PrimarySilenceProfile],
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

pub fn derive_adaptive_alert_variances_from_profile(
    items: &[silence_profile::PrimarySilenceProfile],
) -> Vec<AdaptiveAlertVariance> {
    let mut out: Vec<AdaptiveAlertVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_alert_variance(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_adaptive_alert_variance(item: &AdaptiveAlertVariance) -> String {
    silence_profile::classify_primary_silence_profile(item.adjusted_exposure())
}
