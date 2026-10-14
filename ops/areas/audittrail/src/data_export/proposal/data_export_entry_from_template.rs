use super::*;

use std::collections::BTreeMap;

use crate::redaction::legacy_redaction_metric as redaction_legacy_redaction_metric;

pub fn summarize_default_redaction_templates_for_entry(
    items: &[redaction_legacy_redaction_metric::DefaultRedactionTemplate],
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

pub fn derive_data_export_entries_from_template(
    items: &[redaction_legacy_redaction_metric::DefaultRedactionTemplate],
) -> Vec<DataExportEntry> {
    let mut out: Vec<DataExportEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_data_export_entry(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_data_export_entry(item: &DataExportEntry) -> String {
    redaction_legacy_redaction_metric::classify_default_redaction_template(item.adjusted_exposure())
}
