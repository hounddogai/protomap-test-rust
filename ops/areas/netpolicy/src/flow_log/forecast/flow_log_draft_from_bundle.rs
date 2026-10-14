use super::*;

use std::collections::BTreeMap;

use crate::allowlist::bundle as allowlist_bundle;

pub fn summarize_strict_allowlist_bundles_for_draft(
    items: &[allowlist_bundle::StrictAllowlistBundle],
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

pub fn derive_flow_log_drafts_from_bundle(items: &[allowlist_bundle::StrictAllowlistBundle]) -> Vec<FlowLogDraft> {
    let mut out: Vec<FlowLogDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_draft(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_flow_log_draft(item: &FlowLogDraft) -> String {
    allowlist_bundle::strict_allowlist_bundle_bucket(item.projected_volume())
}
