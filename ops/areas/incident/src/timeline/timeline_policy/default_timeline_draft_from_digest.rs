use super::*;

use std::collections::BTreeMap;

use crate::page::request as page_request;

pub fn summarize_page_digests_for_draft(items: &[page_request::PageDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_default_timeline_drafts_from_digest(items: &[page_request::PageDigest]) -> Vec<DefaultTimelineDraft> {
    let mut out: Vec<DefaultTimelineDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_default_timeline_draft(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_default_timeline_draft(item: &DefaultTimelineDraft) -> String {
    page_request::fallback_page_group_bucket(item.net_total())
}
