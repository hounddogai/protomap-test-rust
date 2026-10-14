use super::*;

use std::collections::BTreeMap;

use crate::approval::budget as approval_budget;

pub fn summarize_approval_audits_for_variance(items: &[approval_budget::ApprovalAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_freeze_variances_from_audit(items: &[approval_budget::ApprovalAudit]) -> Vec<FreezeVariance> {
    let mut out: Vec<FreezeVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_variance(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_freeze_variance(item: &FreezeVariance) -> String {
    approval_budget::classify_approval_budget(item.projected_total())
}
