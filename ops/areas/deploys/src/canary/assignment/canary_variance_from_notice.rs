use super::*;

use std::collections::BTreeMap;

use crate::rollback::notice as rollback_notice;

pub fn summarize_rollback_notices_for_variance(items: &[rollback_notice::RollbackNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_canary_variances_from_notice(items: &[rollback_notice::RollbackNotice]) -> Vec<CanaryVariance> {
    let mut out: Vec<CanaryVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_variance(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_canary_variance(item: &CanaryVariance) -> f64 {
    rollback_notice::pending_rollback_notice_weight(9, true)
}
