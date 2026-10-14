use super::*;

use std::collections::BTreeMap;

use crate::audit_event::profile as audit_event_profile;

pub fn summarize_audit_event_drafts_for_digest(
    items: &[audit_event_profile::AuditEventDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_retention_digests_from_draft(items: &[audit_event_profile::AuditEventDraft]) -> Vec<RetentionDigest> {
    let mut out: Vec<RetentionDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_digest(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_retention_digest(item: &RetentionDigest) -> String {
    audit_event_profile::audit_event_draft_bucket(item.weighted_volume())
}
