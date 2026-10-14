use super::*;

use std::collections::BTreeMap;

use crate::health_check::budget as health_check_budget;

pub fn summarize_health_check_budgets_for_audit(
    items: &[health_check_budget::HealthCheckBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_ttl_policy_audits_from_budget(items: &[health_check_budget::HealthCheckBudget]) -> Vec<TtlPolicyAudit> {
    let mut out: Vec<TtlPolicyAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_audit(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_ttl_policy_audit(item: &TtlPolicyAudit) -> String {
    health_check_budget::health_check_budget_bucket(item.effective_total())
}
