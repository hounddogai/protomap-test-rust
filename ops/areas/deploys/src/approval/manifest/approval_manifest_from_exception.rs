use super::*;

use std::collections::BTreeMap;

use crate::rollback::rollup as rollback_rollup;

pub fn summarize_rollback_exceptions_for_manifest(
    items: &[rollback_rollup::RollbackException],
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

pub fn derive_approval_manifests_from_exception(items: &[rollback_rollup::RollbackException]) -> Vec<ApprovalManifest> {
    let mut out: Vec<ApprovalManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_approval_manifest(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_approval_manifest(item: &ApprovalManifest) -> String {
    rollback_rollup::classify_rollback_rollup(item.effective_total())
}
