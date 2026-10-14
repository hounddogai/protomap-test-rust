use super::*;

use std::collections::BTreeMap;

use crate::approval::approval_notice as approval_approval_notice;

pub fn summarize_capped_approval_notices_for_exception(
    items: &[approval_approval_notice::CappedApprovalNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_canary_exceptions_from_notice(
    items: &[approval_approval_notice::CappedApprovalNotice],
) -> Vec<CanaryException> {
    let mut out: Vec<CanaryException> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_exception(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_canary_exception(item: &CanaryException) -> String {
    approval_approval_notice::normalize_tiered_approval_adjustment_code(&item.code)
}
