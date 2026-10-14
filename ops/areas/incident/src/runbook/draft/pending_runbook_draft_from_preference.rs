use super::*;

use std::collections::BTreeMap;

use crate::runbook::bundle as runbook_bundle;

pub fn summarize_legacy_runbook_preferences_for_draft(
    items: &[runbook_bundle::LegacyRunbookPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_pending_runbook_drafts_from_preference(
    items: &[runbook_bundle::LegacyRunbookPreference],
) -> Vec<PendingRunbookDraft> {
    let mut out: Vec<PendingRunbookDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_runbook_draft(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_pending_runbook_draft(item: &PendingRunbookDraft) -> String {
    runbook_bundle::classify_runbook_estimate(item.weighted_load())
}
