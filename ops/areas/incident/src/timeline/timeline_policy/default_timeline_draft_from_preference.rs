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

pub fn derive_default_timeline_drafts_from_preference(
    items: &[runbook_bundle::LegacyRunbookPreference],
) -> Vec<DefaultTimelineDraft> {
    let mut out: Vec<DefaultTimelineDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_default_timeline_draft(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_default_timeline_draft(item: &DefaultTimelineDraft) -> String {
    runbook_bundle::classify_runbook_estimate(item.net_total())
}
