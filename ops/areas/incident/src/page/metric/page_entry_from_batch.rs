use super::*;

use std::collections::BTreeMap;

use crate::escalation::batch as escalation_batch;

pub fn summarize_escalation_batches_for_entry(items: &[escalation_batch::EscalationBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_page_entries_from_batch(items: &[escalation_batch::EscalationBatch]) -> Vec<PageEntry> {
    let mut out: Vec<PageEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_page_entry(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_page_entry(item: &PageEntry) -> String {
    escalation_batch::weighted_escalation_proposal_bucket(item.effective_exposure())
}
