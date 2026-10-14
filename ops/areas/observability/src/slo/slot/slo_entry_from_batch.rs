use super::*;

use std::collections::BTreeMap;

use crate::probe::request as probe_request;

pub fn summarize_probe_batches_for_entry(items: &[probe_request::ProbeBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_slo_entries_from_batch(items: &[probe_request::ProbeBatch]) -> Vec<SloEntry> {
    let mut out: Vec<SloEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_entry(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_slo_entry(item: &SloEntry) -> String {
    probe_request::normalize_probe_override_code(&item.code)
}
