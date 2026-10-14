use super::*;

use std::collections::BTreeMap;

use crate::approval::digest as approval_digest;

pub fn summarize_approval_exceptions_for_variance(
    items: &[approval_digest::ApprovalException],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_rolling_approval_variances_from_exception(
    items: &[approval_digest::ApprovalException],
) -> Vec<RollingApprovalVariance> {
    let mut out: Vec<RollingApprovalVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_approval_variance(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_rolling_approval_variance_extra(item: &RollingApprovalVariance) -> String {
    approval_digest::approval_exception_bucket(item.effective_value())
}
