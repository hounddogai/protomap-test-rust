use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::review as budget_burn_review;

pub fn summarize_budget_burn_proposals_for_ledger(
    items: &[budget_burn_review::BudgetBurnProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_budget_burn_ledgers_from_proposal(
    items: &[budget_burn_review::BudgetBurnProposal],
) -> Vec<BudgetBurnLedger> {
    let mut out: Vec<BudgetBurnLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_ledger(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_budget_burn_ledger(item: &BudgetBurnLedger) -> f64 {
    budget_burn_review::budget_burn_review_weight(7, true)
}
