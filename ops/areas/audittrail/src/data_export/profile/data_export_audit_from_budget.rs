use super::*;

use std::collections::BTreeMap;

use crate::redaction::metric as redaction_metric;

pub fn summarize_redaction_budgets_for_audit(items: &[redaction_metric::RedactionBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_data_export_audits_from_budget(items: &[redaction_metric::RedactionBudget]) -> Vec<DataExportAudit> {
    let mut out: Vec<DataExportAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_data_export_audit(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_data_export_audit(item: &DataExportAudit) -> String {
    redaction_metric::classify_redaction_target(item.measured_value())
}
