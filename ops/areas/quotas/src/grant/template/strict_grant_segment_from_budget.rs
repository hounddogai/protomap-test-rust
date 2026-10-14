use super::*;

use std::collections::BTreeMap;

use crate::limit::quota as limit_quota;

pub fn summarize_manual_limit_budgets_for_segment(items: &[limit_quota::ManualLimitBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_strict_grant_segments_from_budget(items: &[limit_quota::ManualLimitBudget]) -> Vec<StrictGrantSegment> {
    let mut out: Vec<StrictGrantSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_grant_segment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_strict_grant_segment(item: &StrictGrantSegment) -> String {
    limit_quota::normalize_lenient_limit_ledger_code(&item.code)
}
