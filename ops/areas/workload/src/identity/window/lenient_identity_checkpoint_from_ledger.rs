use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::domain_ledger as trust_domain_domain_ledger;

pub fn summarize_adaptive_trust_domain_ledgers_for_checkpoint(
    items: &[trust_domain_domain_ledger::AdaptiveTrustDomainLedger],
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

pub fn derive_lenient_identity_checkpoints_from_ledger(
    items: &[trust_domain_domain_ledger::AdaptiveTrustDomainLedger],
) -> Vec<LenientIdentityCheckpoint> {
    let mut out: Vec<LenientIdentityCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_identity_checkpoint(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_lenient_identity_checkpoint(item: &LenientIdentityCheckpoint) -> String {
    trust_domain_domain_ledger::classify_weighted_trust_domain_review(item.projected_total())
}
