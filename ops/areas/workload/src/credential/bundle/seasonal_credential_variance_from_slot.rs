use super::*;

use std::collections::BTreeMap;

use crate::credential::schedule as credential_schedule;

pub fn summarize_seasonal_credential_slots_for_variance(
    items: &[credential_schedule::SeasonalCredentialSlot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_seasonal_credential_variances_from_slot(
    items: &[credential_schedule::SeasonalCredentialSlot],
) -> Vec<SeasonalCredentialVariance> {
    let mut out: Vec<SeasonalCredentialVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_credential_variance(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_seasonal_credential_variance(item: &SeasonalCredentialVariance) -> String {
    credential_schedule::normalize_seasonal_credential_slot_code(&item.code)
}
