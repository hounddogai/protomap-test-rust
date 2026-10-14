use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::burn_claim as budget_burn_burn_claim;

pub fn summarize_tiered_budget_burn_claims_for_adjustment(
    items: &[budget_burn_burn_claim::TieredBudgetBurnClaim],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_probe_adjustments_from_claim(
    items: &[budget_burn_burn_claim::TieredBudgetBurnClaim],
) -> Vec<ProbeAdjustment> {
    let mut out: Vec<ProbeAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_adjustment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_probe_adjustment(item: &ProbeAdjustment) -> String {
    budget_burn_burn_claim::tiered_budget_burn_claim_bucket(item.net_total())
}
