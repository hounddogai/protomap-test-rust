use super::*;

use std::collections::BTreeMap;

use crate::runbook::assignment as runbook_assignment;

pub fn summarize_weighted_runbook_agreements_for_manifest(
    items: &[runbook_assignment::WeightedRunbookAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_postmortem_manifests_from_agreement(
    items: &[runbook_assignment::WeightedRunbookAgreement],
) -> Vec<PostmortemManifest> {
    let mut out: Vec<PostmortemManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_manifest(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_postmortem_manifest_extra(item: &PostmortemManifest) -> String {
    runbook_assignment::normalize_runbook_assignment_code(&item.code)
}
