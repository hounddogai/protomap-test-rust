use super::*;

use std::collections::BTreeMap;

use crate::release::batch as release_batch;

pub fn summarize_release_tiers_for_draft(items: &[release_batch::ReleaseTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_rollback_drafts_from_tier(items: &[release_batch::ReleaseTier]) -> Vec<RollbackDraft> {
    let mut out: Vec<RollbackDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_draft(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_rollback_draft(item: &RollbackDraft) -> f64 {
    release_batch::release_digest_weight(8, true)
}
