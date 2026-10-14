use super::*;

use std::collections::BTreeMap;

use crate::probe::agreement as probe_agreement;

pub fn summarize_pending_probe_requests_for_policy(
    items: &[probe_agreement::PendingProbeRequest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_budget_burn_policies_from_request(
    items: &[probe_agreement::PendingProbeRequest],
) -> Vec<BudgetBurnPolicy> {
    let mut out: Vec<BudgetBurnPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_policy(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_budget_burn_policy_extra(item: &BudgetBurnPolicy) -> f64 {
    probe_agreement::pending_probe_request_weight(6, true)
}
