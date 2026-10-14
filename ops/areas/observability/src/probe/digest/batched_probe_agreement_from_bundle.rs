use super::*;

use std::collections::BTreeMap;

use crate::dashboard::bundle as dashboard_bundle;

pub fn summarize_dashboard_bundles_for_agreement(items: &[dashboard_bundle::DashboardBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_batched_probe_agreements_from_bundle(
    items: &[dashboard_bundle::DashboardBundle],
) -> Vec<BatchedProbeAgreement> {
    let mut out: Vec<BatchedProbeAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_probe_agreement(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_batched_probe_agreement(item: &BatchedProbeAgreement) -> String {
    dashboard_bundle::classify_dashboard_profile(item.adjusted_volume())
}
