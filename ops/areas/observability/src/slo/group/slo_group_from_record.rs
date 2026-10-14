use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::proposal as budget_burn_proposal;

pub fn summarize_budget_burn_records_for_group(
    items: &[budget_burn_proposal::BudgetBurnRecord],
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

pub fn derive_slo_groups_from_record(items: &[budget_burn_proposal::BudgetBurnRecord]) -> Vec<SloGroup> {
    let mut out: Vec<SloGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_group(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_slo_group(item: &SloGroup) -> String {
    budget_burn_proposal::budget_burn_rollup_bucket(item.net_total())
}
