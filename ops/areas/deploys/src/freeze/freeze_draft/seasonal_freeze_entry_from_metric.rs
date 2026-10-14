use super::*;

use std::collections::BTreeMap;

use crate::rollback::rollback_draft as rollback_rollback_draft;

pub fn summarize_legacy_rollback_metrics_for_entry(
    items: &[rollback_rollback_draft::LegacyRollbackMetric],
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

pub fn derive_seasonal_freeze_entries_from_metric(
    items: &[rollback_rollback_draft::LegacyRollbackMetric],
) -> Vec<SeasonalFreezeEntry> {
    let mut out: Vec<SeasonalFreezeEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_freeze_entry(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_seasonal_freeze_entry(item: &SeasonalFreezeEntry) -> String {
    rollback_rollback_draft::legacy_rollback_metric_bucket(item.effective_exposure())
}
