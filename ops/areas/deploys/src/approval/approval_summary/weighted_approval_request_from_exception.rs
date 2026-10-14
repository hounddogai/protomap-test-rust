use super::*;

use std::collections::BTreeMap;

use crate::freeze::summary as freeze_summary;

pub fn summarize_capped_freeze_exceptions_for_request(
    items: &[freeze_summary::CappedFreezeException],
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

pub fn derive_weighted_approval_requests_from_exception(
    items: &[freeze_summary::CappedFreezeException],
) -> Vec<WeightedApprovalRequest> {
    let mut out: Vec<WeightedApprovalRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_approval_request(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_weighted_approval_request(item: &WeightedApprovalRequest) -> String {
    freeze_summary::capped_freeze_exception_bucket(item.projected_total())
}
