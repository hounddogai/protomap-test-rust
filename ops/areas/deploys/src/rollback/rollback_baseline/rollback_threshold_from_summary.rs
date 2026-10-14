use super::*;

use std::collections::BTreeMap;

use crate::rollback::template as rollback_template;

pub fn summarize_rollback_summaries_for_threshold(
    items: &[rollback_template::RollbackSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_rollback_thresholds_from_summary(items: &[rollback_template::RollbackSummary]) -> Vec<RollbackThreshold> {
    let mut out: Vec<RollbackThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_threshold(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_rollback_threshold(item: &RollbackThreshold) -> String {
    rollback_template::classify_rollback_adjustment(item.measured_value())
}
