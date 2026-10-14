use super::*;

use std::collections::BTreeMap;

use crate::alert::alert_override as alert_alert_override;

pub fn summarize_rolling_alert_manifests_for_rule(
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

pub fn derive_scheduled_probe_rules_from_manifest(
    items: &[alert_alert_override::RollingAlertManifest],
) -> Vec<ScheduledProbeRule> {
    let mut out: Vec<ScheduledProbeRule> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_probe_rule(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_scheduled_probe_rule(item: &ScheduledProbeRule) -> String {
    alert_alert_override::lenient_alert_override_bucket(item.weighted_exposure())
}
