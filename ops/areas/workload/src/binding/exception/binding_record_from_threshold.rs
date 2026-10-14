use super::*;

use std::collections::BTreeMap;

use crate::binding::threshold as binding_threshold;

pub fn summarize_batched_binding_thresholds_for_record(
    items: &[binding_threshold::BatchedBindingThreshold],
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

pub fn derive_binding_records_from_threshold(
    items: &[binding_threshold::BatchedBindingThreshold],
) -> Vec<BindingRecord> {
    let mut out: Vec<BindingRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_record(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_binding_record(item: &BindingRecord) -> String {
    binding_threshold::batched_binding_threshold_bucket(item.effective_volume())
}
