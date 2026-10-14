use super::*;

use std::collections::BTreeMap;

use crate::redaction::adjustment as redaction_adjustment;

pub fn summarize_regional_redaction_groups_for_audit(
    items: &[redaction_adjustment::RegionalRedactionGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_retention_audits_from_group(
    items: &[redaction_adjustment::RegionalRedactionGroup],
) -> Vec<RetentionAudit> {
    let mut out: Vec<RetentionAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_audit(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_retention_audit(item: &RetentionAudit) -> String {
    redaction_adjustment::classify_redaction_adjustment(item.effective_exposure())
}
