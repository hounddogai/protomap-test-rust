use super::*;

use std::collections::BTreeMap;

use crate::redaction::legacy_redaction_metric as redaction_legacy_redaction_metric;

pub fn summarize_manual_redaction_bundles_for_slot(
    items: &[redaction_legacy_redaction_metric::ManualRedactionBundle],
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

pub fn derive_manual_query_slots_from_bundle(
    items: &[redaction_legacy_redaction_metric::ManualRedactionBundle],
) -> Vec<ManualQuerySlot> {
    let mut out: Vec<ManualQuerySlot> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_query_slot(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_manual_query_slot(item: &ManualQuerySlot) -> String {
    redaction_legacy_redaction_metric::normalize_manual_redaction_bundle_code(&item.code)
}
