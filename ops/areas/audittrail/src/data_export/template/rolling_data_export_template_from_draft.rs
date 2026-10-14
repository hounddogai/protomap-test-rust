use super::*;

use std::collections::BTreeMap;

use crate::redaction::assignment as redaction_assignment;

pub fn summarize_batched_redaction_drafts_for_template(
    items: &[redaction_assignment::BatchedRedactionDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_rolling_data_export_templates_from_draft(
    items: &[redaction_assignment::BatchedRedactionDraft],
) -> Vec<RollingDataExportTemplate> {
    let mut out: Vec<RollingDataExportTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_data_export_template(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_rolling_data_export_template(item: &RollingDataExportTemplate) -> f64 {
    redaction_assignment::batched_redaction_manifest_weight(4, true)
}
