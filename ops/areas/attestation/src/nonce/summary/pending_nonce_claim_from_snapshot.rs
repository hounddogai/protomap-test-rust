use super::*;

use std::collections::BTreeMap;

use crate::quote::forecast as quote_forecast;

pub fn summarize_quote_snapshots_for_claim(items: &[quote_forecast::QuoteSnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_pending_nonce_claims_from_snapshot(items: &[quote_forecast::QuoteSnapshot]) -> Vec<PendingNonceClaim> {
    let mut out: Vec<PendingNonceClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_nonce_claim(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_pending_nonce_claim(item: &PendingNonceClaim) -> String {
    quote_forecast::quote_snapshot_bucket(item.measured_value())
}
