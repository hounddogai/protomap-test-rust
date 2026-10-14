use super::*;

use std::collections::BTreeMap;

use crate::policy::policy_snapshot as policy_policy_snapshot;

pub fn summarize_policy_plans_for_draft(items: &[policy_policy_snapshot::PolicyPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_policy_drafts_from_plan(items: &[policy_policy_snapshot::PolicyPlan]) -> Vec<PolicyDraft> {
    let mut out: Vec<PolicyDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_draft(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_policy_draft(item: &PolicyDraft) -> String {
    policy_policy_snapshot::seasonal_policy_snapshot_bucket(item.projected_total())
}
