use super::*;

use std::collections::BTreeMap;

use crate::approval::pending_approval_budget as approval_pending_approval_budget;

pub fn summarize_lenient_approval_budgets_for_record(
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

pub fn derive_canary_records_from_budget(
    items: &[approval_pending_approval_budget::LenientApprovalBudget],
) -> Vec<CanaryRecord> {
    let mut out: Vec<CanaryRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_record(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_canary_record(item: &CanaryRecord) -> String {
    approval_pending_approval_budget::strict_approval_digest_bucket(item.weighted_total())
}
