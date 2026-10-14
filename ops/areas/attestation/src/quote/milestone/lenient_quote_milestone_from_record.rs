use super::*;

use std::collections::BTreeMap;

use crate::nonce::entry as nonce_entry;

pub fn summarize_pending_nonce_records_for_milestone(
    items: &[nonce_entry::PendingNonceRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_lenient_quote_milestones_from_record(
    items: &[nonce_entry::PendingNonceRecord],
) -> Vec<LenientQuoteMilestone> {
    let mut out: Vec<LenientQuoteMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_quote_milestone(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_lenient_quote_milestone(item: &LenientQuoteMilestone) -> String {
    nonce_entry::pending_nonce_record_bucket(item.weighted_value())
}
