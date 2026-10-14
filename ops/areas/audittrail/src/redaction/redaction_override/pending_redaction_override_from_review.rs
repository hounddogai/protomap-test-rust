use super::*;

use std::collections::BTreeMap;

use crate::sink::ledger as sink_ledger;

pub fn summarize_scheduled_sink_reviews_for_override(
    items: &[sink_ledger::ScheduledSinkReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_pending_redaction_overrides_from_review(
    items: &[sink_ledger::ScheduledSinkReview],
) -> Vec<PendingRedactionOverride> {
    let mut out: Vec<PendingRedactionOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_redaction_override(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_pending_redaction_override(item: &PendingRedactionOverride) -> String {
    sink_ledger::classify_global_sink_slot(item.net_volume())
}
