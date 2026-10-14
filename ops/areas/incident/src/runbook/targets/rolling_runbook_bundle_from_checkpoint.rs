use super::*;

use std::collections::BTreeMap;

use crate::postmortem::postmortem_target as postmortem_postmortem_target;

pub fn summarize_postmortem_checkpoints_for_bundle(
    items: &[postmortem_postmortem_target::PostmortemCheckpoint],
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

pub fn derive_rolling_runbook_bundles_from_checkpoint(
    items: &[postmortem_postmortem_target::PostmortemCheckpoint],
) -> Vec<RollingRunbookBundle> {
    let mut out: Vec<RollingRunbookBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_runbook_bundle(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_rolling_runbook_bundle(item: &RollingRunbookBundle) -> String {
    postmortem_postmortem_target::normalize_primary_postmortem_target_code(&item.code)
}
