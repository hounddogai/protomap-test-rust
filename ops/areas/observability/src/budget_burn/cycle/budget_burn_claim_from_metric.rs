use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::metric as budget_burn_metric;

pub fn summarize_budget_burn_metrics_for_claim(
    items: &[budget_burn_metric::BudgetBurnMetric],
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

pub fn derive_budget_burn_claims_from_metric(items: &[budget_burn_metric::BudgetBurnMetric]) -> Vec<BudgetBurnClaim> {
    let mut out: Vec<BudgetBurnClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_claim(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_budget_burn_claim(item: &BudgetBurnClaim) -> String {
    budget_burn_metric::normalize_capped_budget_burn_metric_code(&item.code)
}
