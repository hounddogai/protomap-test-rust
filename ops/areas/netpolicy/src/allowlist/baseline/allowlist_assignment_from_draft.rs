use super::*;

use std::collections::BTreeMap;

use crate::allowlist::digest as allowlist_digest;

pub fn summarize_allowlist_drafts_for_assignment(items: &[allowlist_digest::AllowlistDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_allowlist_assignments_from_draft(items: &[allowlist_digest::AllowlistDraft]) -> Vec<AllowlistAssignment> {
    let mut out: Vec<AllowlistAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_assignment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_allowlist_assignment(item: &AllowlistAssignment) -> f64 {
    allowlist_digest::allowlist_draft_weight(7, true)
}
