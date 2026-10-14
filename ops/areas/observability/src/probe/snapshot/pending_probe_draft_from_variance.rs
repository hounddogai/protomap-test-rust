use super::*;

use std::collections::BTreeMap;

use crate::probe::probe_request as probe_probe_request;

pub fn summarize_probe_variances_for_draft(items: &[probe_probe_request::ProbeVariance]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_pending_probe_drafts_from_variance(
    items: &[probe_probe_request::ProbeVariance],
) -> Vec<PendingProbeDraft> {
    let mut out: Vec<PendingProbeDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_probe_draft(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_pending_probe_draft(item: &PendingProbeDraft) -> String {
    probe_probe_request::normalize_manual_probe_request_code(&item.code)
}
