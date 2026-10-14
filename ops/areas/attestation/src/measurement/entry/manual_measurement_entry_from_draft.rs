use super::*;

use std::collections::BTreeMap;

use crate::policy::template as policy_template;

pub fn summarize_policy_drafts_for_entry(items: &[policy_template::PolicyDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_manual_measurement_entries_from_draft(
    items: &[policy_template::PolicyDraft],
) -> Vec<ManualMeasurementEntry> {
    let mut out: Vec<ManualMeasurementEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_measurement_entry(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_manual_measurement_entry(item: &ManualMeasurementEntry) -> String {
    policy_template::rolling_policy_rollup_bucket(item.adjusted_volume())
}
