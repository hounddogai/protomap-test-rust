use super::*;

use std::collections::BTreeMap;

use crate::alert::manifest as alert_manifest;

pub fn summarize_alert_entries_for_batch(items: &[alert_manifest::AlertEntry]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_probe_batches_from_entry(items: &[alert_manifest::AlertEntry]) -> Vec<ProbeBatch> {
    let mut out: Vec<ProbeBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_batch(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_probe_batch(item: &ProbeBatch) -> String {
    alert_manifest::classify_alert_slot(item.weighted_value())
}
