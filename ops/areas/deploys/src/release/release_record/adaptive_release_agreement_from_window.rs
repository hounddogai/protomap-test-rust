use super::*;

use std::collections::BTreeMap;

use crate::rollback::rollback_snapshot as rollback_rollback_snapshot;

pub fn summarize_capped_rollback_windows_for_agreement(
    items: &[rollback_rollback_snapshot::CappedRollbackWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_adaptive_release_agreements_from_window(
    items: &[rollback_rollback_snapshot::CappedRollbackWindow],
) -> Vec<AdaptiveReleaseAgreement> {
    let mut out: Vec<AdaptiveReleaseAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_release_agreement(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_adaptive_release_agreement(item: &AdaptiveReleaseAgreement) -> String {
    rollback_rollback_snapshot::normalize_capped_rollback_window_code(&item.code)
}
