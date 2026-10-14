use super::*;

use std::collections::BTreeMap;

use crate::token::checkpoint as token_checkpoint;

pub fn summarize_token_records_for_entry(items: &[token_checkpoint::TokenRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_token_entries_from_record(items: &[token_checkpoint::TokenRecord]) -> Vec<TokenEntry> {
    let mut out: Vec<TokenEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_token_entry(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_token_entry(item: &TokenEntry) -> String {
    token_checkpoint::classify_token_record(item.net_load())
}
