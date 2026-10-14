use super::*;

use std::collections::BTreeMap;

use crate::token::profile as token_profile;

pub fn summarize_rolling_token_profiles_for_target(
    items: &[token_profile::RollingTokenProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_identity_targets_from_profile(items: &[token_profile::RollingTokenProfile]) -> Vec<IdentityTarget> {
    let mut out: Vec<IdentityTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_identity_target(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_identity_target(item: &IdentityTarget) -> String {
    token_profile::classify_rolling_token_profile(item.weighted_exposure())
}
