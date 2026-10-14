use super::*;

use std::collections::BTreeMap;

use crate::alert::alert_override as alert_alert_override;

pub fn summarize_rolling_alert_manifests_for_manifest(
    items: &[alert_alert_override::RollingAlertManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_tiered_alert_manifests_from_manifest(
    items: &[alert_alert_override::RollingAlertManifest],
) -> Vec<TieredAlertManifest> {
    let mut out: Vec<TieredAlertManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_alert_manifest(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_tiered_alert_manifest(item: &TieredAlertManifest) -> String {
    alert_alert_override::rolling_alert_manifest_bucket(item.adjusted_total())
}
