use super::*;

use std::collections::BTreeMap;

use crate::silence::silence_metric as silence_silence_metric;

pub fn summarize_strict_silence_preferences_for_manifest(
    items: &[silence_silence_metric::StrictSilencePreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_rolling_alert_manifests_from_preference(
    items: &[silence_silence_metric::StrictSilencePreference],
) -> Vec<RollingAlertManifest> {
    let mut out: Vec<RollingAlertManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_alert_manifest(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_rolling_alert_manifest(item: &RollingAlertManifest) -> String {
    silence_silence_metric::classify_fallback_silence_bundle(item.projected_total())
}
