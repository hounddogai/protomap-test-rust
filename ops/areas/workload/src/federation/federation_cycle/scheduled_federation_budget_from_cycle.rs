use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::cycle as trust_domain_cycle;

pub fn summarize_trust_domain_cycles_for_budget(
    items: &[trust_domain_cycle::TrustDomainCycle],
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

pub fn derive_scheduled_federation_budgets_from_cycle(
    items: &[trust_domain_cycle::TrustDomainCycle],
) -> Vec<ScheduledFederationBudget> {
    let mut out: Vec<ScheduledFederationBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_federation_budget(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_scheduled_federation_budget(item: &ScheduledFederationBudget) -> String {
    trust_domain_cycle::legacy_trust_domain_reservation_bucket(item.projected_volume())
}
