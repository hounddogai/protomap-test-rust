use super::*;

use std::collections::BTreeMap;

use crate::credential::score as credential_score;

pub fn summarize_credential_snapshots_for_budget(
    items: &[credential_score::CredentialSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_trust_domain_budgets_from_snapshot(
    items: &[credential_score::CredentialSnapshot],
) -> Vec<TrustDomainBudget> {
    let mut out: Vec<TrustDomainBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_budget(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_trust_domain_budget(item: &TrustDomainBudget) -> String {
    credential_score::normalize_credential_snapshot_code(&item.code)
}
