use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::profile as trust_domain_profile;

pub fn summarize_trust_domain_baselines_for_ledger(
    items: &[trust_domain_profile::TrustDomainBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_trust_domain_ledgers_from_baseline(
    items: &[trust_domain_profile::TrustDomainBaseline],
) -> Vec<TrustDomainLedger> {
    let mut out: Vec<TrustDomainLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_ledger(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_trust_domain_ledger(item: &TrustDomainLedger) -> String {
    trust_domain_profile::classify_trust_domain_profile(item.weighted_total())
}
