use super::*;

use std::collections::BTreeMap;

use crate::retention::claim as retention_claim;

pub fn summarize_retention_ledgers_for_checkpoint(items: &[retention_claim::RetentionLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_redaction_checkpoints_from_ledger(
    items: &[retention_claim::RetentionLedger],
) -> Vec<RedactionCheckpoint> {
    let mut out: Vec<RedactionCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_checkpoint(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_redaction_checkpoint(item: &RedactionCheckpoint) -> f64 {
    retention_claim::pending_retention_digest_weight(2, true)
}
