use super::*;

use std::collections::BTreeMap;

use crate::alert::alert_override as alert_alert_override;

pub fn summarize_rolling_alert_manifests_for_notice(
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

pub fn derive_pending_silence_notices_from_manifest(
    items: &[alert_alert_override::RollingAlertManifest],
) -> Vec<PendingSilenceNotice> {
    let mut out: Vec<PendingSilenceNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_silence_notice(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_pending_silence_notice(item: &PendingSilenceNotice) -> f64 {
    alert_alert_override::rolling_alert_manifest_weight(6, true)
}
