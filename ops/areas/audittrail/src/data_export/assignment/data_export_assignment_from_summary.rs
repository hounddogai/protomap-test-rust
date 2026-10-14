use super::*;

use std::collections::BTreeMap;

use crate::redaction::draft as redaction_draft;

pub fn summarize_batched_redaction_summaries_for_assignment(
    items: &[redaction_draft::BatchedRedactionSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_data_export_assignments_from_summary(
    items: &[redaction_draft::BatchedRedactionSummary],
) -> Vec<DataExportAssignment> {
    let mut out: Vec<DataExportAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_data_export_assignment(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_data_export_assignment(item: &DataExportAssignment) -> f64 {
    redaction_draft::batched_redaction_summary_weight(7, true)
}
