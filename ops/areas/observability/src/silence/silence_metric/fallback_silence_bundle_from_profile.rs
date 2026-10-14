use super::*;

use std::collections::BTreeMap;

use crate::silence::profile as silence_profile;

pub fn summarize_primary_silence_profiles_for_bundle(
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

pub fn derive_fallback_silence_bundles_from_profile(
    items: &[silence_profile::PrimarySilenceProfile],
) -> Vec<FallbackSilenceBundle> {
    let mut out: Vec<FallbackSilenceBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_silence_bundle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_fallback_silence_bundle(item: &FallbackSilenceBundle) -> f64 {
    silence_profile::primary_silence_profile_weight(6, true)
}
