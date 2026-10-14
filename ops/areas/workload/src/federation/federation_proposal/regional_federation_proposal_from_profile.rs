use super::*;

use std::collections::BTreeMap;

use crate::token::profile as token_profile;

pub fn summarize_rolling_token_profiles_for_proposal(
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

pub fn derive_regional_federation_proposals_from_profile(
    items: &[token_profile::RollingTokenProfile],
) -> Vec<RegionalFederationProposal> {
    let mut out: Vec<RegionalFederationProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_federation_proposal(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_regional_federation_proposal(item: &RegionalFederationProposal) -> String {
    token_profile::classify_rolling_token_profile(item.net_volume())
}
