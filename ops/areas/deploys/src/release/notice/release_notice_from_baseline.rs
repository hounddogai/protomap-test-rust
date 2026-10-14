use super::*;

use std::collections::BTreeMap;

use crate::rollback::baseline as rollback_baseline;

pub fn summarize_rollback_baselines_for_notice(items: &[rollback_baseline::RollbackBaseline]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_release_notices_from_baseline(items: &[rollback_baseline::RollbackBaseline]) -> Vec<ReleaseNotice> {
    let mut out: Vec<ReleaseNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_release_notice(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_release_notice(item: &ReleaseNotice) -> String {
    rollback_baseline::rollback_preference_bucket(item.net_volume())
}
