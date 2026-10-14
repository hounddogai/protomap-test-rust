use super::*;

use std::collections::BTreeMap;

use crate::approval::approval_quota as approval_approval_quota;

pub fn summarize_approval_requests_for_audit(
    items: &[approval_approval_quota::ApprovalRequest],
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

pub fn derive_seasonal_rollback_audits_from_request(
    items: &[approval_approval_quota::ApprovalRequest],
) -> Vec<SeasonalRollbackAudit> {
    let mut out: Vec<SeasonalRollbackAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_rollback_audit(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_seasonal_rollback_audit(item: &SeasonalRollbackAudit) -> String {
    approval_approval_quota::normalize_scheduled_approval_quota_code(&item.code)
}
