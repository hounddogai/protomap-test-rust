use super::*;

use std::collections::BTreeMap;

use crate::delegation::delegation_ledger as delegation_delegation_ledger;

pub fn summarize_strict_delegation_targets_for_profile(
    items: &[delegation_delegation_ledger::StrictDelegationTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_primary_ttl_policy_profiles_from_target(
    items: &[delegation_delegation_ledger::StrictDelegationTarget],
) -> Vec<PrimaryTtlPolicyProfile> {
    let mut out: Vec<PrimaryTtlPolicyProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_ttl_policy_profile(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_primary_ttl_policy_profile_extra(item: &PrimaryTtlPolicyProfile) -> String {
    delegation_delegation_ledger::classify_delegation_preference(item.measured_value())
}
