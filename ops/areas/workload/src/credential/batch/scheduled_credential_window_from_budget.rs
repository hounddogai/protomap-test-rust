use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::snapshot as trust_domain_snapshot;

pub fn summarize_scheduled_trust_domain_budgets_for_window(
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

pub fn derive_scheduled_credential_windows_from_budget(
    items: &[trust_domain_snapshot::ScheduledTrustDomainBudget],
) -> Vec<ScheduledCredentialWindow> {
    let mut out: Vec<ScheduledCredentialWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_credential_window(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_scheduled_credential_window(item: &ScheduledCredentialWindow) -> String {
    trust_domain_snapshot::classify_scheduled_trust_domain_budget(item.projected_total())
}
