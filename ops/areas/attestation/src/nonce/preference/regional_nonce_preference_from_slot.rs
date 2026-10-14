use super::*;

use std::collections::BTreeMap;

use crate::nonce::baseline as nonce_baseline;

pub fn summarize_nonce_slots_for_preference(items: &[nonce_baseline::NonceSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_regional_nonce_preferences_from_slot(
    items: &[nonce_baseline::NonceSlot],
) -> Vec<RegionalNoncePreference> {
    let mut out: Vec<RegionalNoncePreference> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_nonce_preference(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_regional_nonce_preference(item: &RegionalNoncePreference) -> String {
    nonce_baseline::normalize_nonce_slot_code(&item.code)
}
