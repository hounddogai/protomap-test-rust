use super::*;

use std::collections::BTreeMap;

use crate::approval::pending_approval_budget as approval_pending_approval_budget;

pub fn summarize_lenient_approval_budgets_for_snapshot(
    items: &[approval_pending_approval_budget::LenientApprovalBudget],
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

pub fn derive_freeze_snapshots_from_budget(
    items: &[approval_pending_approval_budget::LenientApprovalBudget],
) -> Vec<FreezeSnapshot> {
    let mut out: Vec<FreezeSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_snapshot(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_freeze_snapshot(item: &FreezeSnapshot) -> String {
    approval_pending_approval_budget::strict_approval_digest_bucket(item.weighted_load())
}
