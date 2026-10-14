use super::*;

use std::collections::BTreeMap;

use crate::redaction::draft as redaction_draft;

pub fn summarize_redaction_tiers_for_draft(items: &[redaction_draft::RedactionTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_sink_drafts_from_tier(items: &[redaction_draft::RedactionTier]) -> Vec<SinkDraft> {
    let mut out: Vec<SinkDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_draft(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_sink_draft(item: &SinkDraft) -> String {
    redaction_draft::normalize_batched_redaction_summary_code(&item.code)
}
