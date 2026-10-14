use super::*;

use std::collections::BTreeMap;

use crate::escalation::profile as escalation_profile;

pub fn summarize_default_escalation_profiles_for_manifest(
    items: &[escalation_profile::DefaultEscalationProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_runbook_manifests_from_profile(
    items: &[escalation_profile::DefaultEscalationProfile],
) -> Vec<RunbookManifest> {
    let mut out: Vec<RunbookManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_runbook_manifest(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_runbook_manifest(item: &RunbookManifest) -> String {
    escalation_profile::normalize_escalation_request_code(&item.code)
}
