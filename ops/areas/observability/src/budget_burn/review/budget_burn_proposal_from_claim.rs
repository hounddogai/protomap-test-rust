use super::*;

use std::collections::BTreeMap;

use crate::alert::alert_cycle as alert_alert_cycle;

pub fn summarize_alert_claims_for_proposal(items: &[alert_alert_cycle::AlertClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_budget_burn_proposals_from_claim(items: &[alert_alert_cycle::AlertClaim]) -> Vec<BudgetBurnProposal> {
    let mut out: Vec<BudgetBurnProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_proposal(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_budget_burn_proposal(item: &BudgetBurnProposal) -> f64 {
    alert_alert_cycle::alert_record_weight(3, true)
}
