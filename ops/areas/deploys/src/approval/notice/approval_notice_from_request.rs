use super::*;

use std::collections::BTreeMap;

use crate::freeze::rollup as freeze_rollup;

pub fn summarize_freeze_requests_for_notice(items: &[freeze_rollup::FreezeRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_approval_notices_from_request(items: &[freeze_rollup::FreezeRequest]) -> Vec<ApprovalNotice> {
    let mut out: Vec<ApprovalNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_approval_notice(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_approval_notice(item: &ApprovalNotice) -> String {
    freeze_rollup::normalize_freeze_plan_code(&item.code)
}
