use super::*;

use std::collections::BTreeMap;

use crate::identity::digest as identity_digest;

pub fn summarize_scheduled_identity_rules_for_profile(
    items: &[identity_digest::ScheduledIdentityRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_binding_profiles_from_rule(items: &[identity_digest::ScheduledIdentityRule]) -> Vec<BindingProfile> {
    let mut out: Vec<BindingProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_profile(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_binding_profile(item: &BindingProfile) -> f64 {
    identity_digest::primary_identity_claim_weight(1, true)
}
