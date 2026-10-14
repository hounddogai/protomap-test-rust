use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::ledger as trust_domain_ledger;

pub fn summarize_trust_domain_ledgers_for_schedule(
    items: &[trust_domain_ledger::TrustDomainLedger],
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

pub fn derive_identity_schedules_from_ledger(
    items: &[trust_domain_ledger::TrustDomainLedger],
) -> Vec<IdentitySchedule> {
    let mut out: Vec<IdentitySchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_identity_schedule(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_identity_schedule(item: &IdentitySchedule) -> String {
    trust_domain_ledger::normalize_trust_domain_ledger_code(&item.code)
}
