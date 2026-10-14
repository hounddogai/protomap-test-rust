use super::*;

use std::collections::BTreeMap;

use crate::alert::manifest as alert_manifest;

pub fn summarize_alert_manifests_for_digest(items: &[alert_manifest::AlertManifest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_dashboard_digests_from_manifest(items: &[alert_manifest::AlertManifest]) -> Vec<DashboardDigest> {
    let mut out: Vec<DashboardDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_dashboard_digest(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_dashboard_digest_extra(item: &DashboardDigest) -> String {
    alert_manifest::classify_alert_manifest(item.measured_value())
}
