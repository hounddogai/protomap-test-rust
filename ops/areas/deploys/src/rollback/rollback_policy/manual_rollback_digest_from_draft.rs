use super::*;

use std::collections::BTreeMap;

use crate::canary::draft as canary_draft;

pub fn summarize_canary_drafts_for_digest(items: &[canary_draft::CanaryDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_manual_rollback_digests_from_draft(items: &[canary_draft::CanaryDraft]) -> Vec<ManualRollbackDigest> {
    let mut out: Vec<ManualRollbackDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_rollback_digest(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_manual_rollback_digest(item: &ManualRollbackDigest) -> f64 {
    canary_draft::canary_draft_weight(1, true)
}
