use super::*;

use std::collections::BTreeMap;

use crate::overage::review as overage_review;

pub fn summarize_capped_overage_snapshots_for_quota(
    items: &[overage_review::CappedOverageSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_grant_quotas_from_snapshot(items: &[overage_review::CappedOverageSnapshot]) -> Vec<GrantQuota> {
    let mut out: Vec<GrantQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_quota(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_grant_quota(item: &GrantQuota) -> String {
    overage_review::normalize_capped_overage_snapshot_code(&item.code)
}
