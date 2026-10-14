use super::*;

use std::collections::BTreeMap;

use crate::runbook::runbook_override as runbook_runbook_override;

pub fn summarize_rolling_runbook_overrides_for_bundle(
    items: &[runbook_runbook_override::RollingRunbookOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_rolling_runbook_bundles_from_override(
    items: &[runbook_runbook_override::RollingRunbookOverride],
) -> Vec<RollingRunbookBundle> {
    let mut out: Vec<RollingRunbookBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_runbook_bundle(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_rolling_runbook_bundle(item: &RollingRunbookBundle) -> String {
    runbook_runbook_override::classify_batched_runbook_summary(item.adjusted_value())
}
