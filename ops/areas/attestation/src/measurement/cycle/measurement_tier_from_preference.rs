use super::*;

use std::collections::BTreeMap;

use crate::nonce::nonce_preference as nonce_nonce_preference;

pub fn summarize_lenient_nonce_preferences_for_tier(
    items: &[nonce_nonce_preference::LenientNoncePreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_measurement_tiers_from_preference(
    items: &[nonce_nonce_preference::LenientNoncePreference],
) -> Vec<MeasurementTier> {
    let mut out: Vec<MeasurementTier> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_tier(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_measurement_tier(item: &MeasurementTier) -> String {
    nonce_nonce_preference::normalize_lenient_nonce_preference_code(&item.code)
}
