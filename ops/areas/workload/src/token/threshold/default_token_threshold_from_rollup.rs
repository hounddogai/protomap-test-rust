use super::*;

use std::collections::BTreeMap;

use crate::credential::rollup as credential_rollup;

pub fn summarize_regional_credential_rollups_for_threshold(
    items: &[credential_rollup::RegionalCredentialRollup],
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

pub fn derive_default_token_thresholds_from_rollup(
    items: &[credential_rollup::RegionalCredentialRollup],
) -> Vec<DefaultTokenThreshold> {
    let mut out: Vec<DefaultTokenThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_default_token_threshold(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_default_token_threshold(item: &DefaultTokenThreshold) -> f64 {
    credential_rollup::weighted_credential_slot_weight(5, true)
}
