use super::*;

use std::collections::BTreeMap;

use crate::redaction::entry as redaction_entry;

pub fn summarize_redaction_claims_for_checkpoint(items: &[redaction_entry::RedactionClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_retention_checkpoints_from_claim(items: &[redaction_entry::RedactionClaim]) -> Vec<RetentionCheckpoint> {
    let mut out: Vec<RetentionCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_checkpoint(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_retention_checkpoint(item: &RetentionCheckpoint) -> f64 {
    redaction_entry::batched_redaction_bundle_weight(8, true)
}
