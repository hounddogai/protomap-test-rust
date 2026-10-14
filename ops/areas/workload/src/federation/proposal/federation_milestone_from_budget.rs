use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::snapshot as trust_domain_snapshot;

pub fn summarize_scheduled_trust_domain_budgets_for_milestone(
    items: &[trust_domain_snapshot::ScheduledTrustDomainBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_federation_milestones_from_budget(
    items: &[trust_domain_snapshot::ScheduledTrustDomainBudget],
) -> Vec<FederationMilestone> {
    let mut out: Vec<FederationMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_federation_milestone(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_federation_milestone(item: &FederationMilestone) -> f64 {
    trust_domain_snapshot::scheduled_trust_domain_budget_weight(5, true)
}
