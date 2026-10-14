use super::*;

use std::collections::BTreeMap;

use crate::delegation::ledger as delegation_ledger;

pub fn summarize_primary_delegation_assignments_for_tier(
    items: &[delegation_ledger::PrimaryDelegationAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_capped_ttl_policy_tiers_from_assignment(
    items: &[delegation_ledger::PrimaryDelegationAssignment],
) -> Vec<CappedTtlPolicyTier> {
    let mut out: Vec<CappedTtlPolicyTier> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_ttl_policy_tier(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_capped_ttl_policy_tier(item: &CappedTtlPolicyTier) -> String {
    delegation_ledger::normalize_delegation_ledger_code(&item.code)
}
