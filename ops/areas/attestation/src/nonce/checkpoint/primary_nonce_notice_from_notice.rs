use super::*;

use std::collections::BTreeMap;

use crate::quote::milestone as quote_milestone;

pub fn summarize_quote_notices_for_notice(items: &[quote_milestone::QuoteNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_primary_nonce_notices_from_notice(items: &[quote_milestone::QuoteNotice]) -> Vec<PrimaryNonceNotice> {
    let mut out: Vec<PrimaryNonceNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_nonce_notice(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_primary_nonce_notice(item: &PrimaryNonceNotice) -> String {
    quote_milestone::lenient_quote_milestone_bucket(item.effective_load())
}
