use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::claim as budget_burn_claim;

pub fn summarize_budget_burn_schedules_for_claim(
    items: &[budget_burn_claim::BudgetBurnSchedule],
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

pub fn derive_probe_claims_from_schedule(items: &[budget_burn_claim::BudgetBurnSchedule]) -> Vec<ProbeClaim> {
    let mut out: Vec<ProbeClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_claim(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_probe_claim_extra(item: &ProbeClaim) -> String {
    budget_burn_claim::global_budget_burn_notice_bucket(item.measured_value())
}
