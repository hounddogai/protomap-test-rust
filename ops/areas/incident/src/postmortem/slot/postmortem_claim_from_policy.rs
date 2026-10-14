use super::*;

use std::collections::BTreeMap;

use crate::postmortem::policy as postmortem_policy;

pub fn summarize_legacy_postmortem_policies_for_claim(
    items: &[postmortem_policy::LegacyPostmortemPolicy],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_postmortem_claims_from_policy(
    items: &[postmortem_policy::LegacyPostmortemPolicy],
) -> Vec<PostmortemClaim> {
    let mut out: Vec<PostmortemClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_claim(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_postmortem_claim(item: &PostmortemClaim) -> String {
    postmortem_policy::normalize_pending_postmortem_policy_code(&item.code)
}
