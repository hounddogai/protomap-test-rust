use super::*;

use std::collections::BTreeMap;

use crate::alert::alert_quota as alert_alert_quota;

pub fn summarize_adaptive_alert_quotas_for_adjustment(
    items: &[alert_alert_quota::AdaptiveAlertQuota],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_legacy_probe_adjustments_from_quota(
    items: &[alert_alert_quota::AdaptiveAlertQuota],
) -> Vec<LegacyProbeAdjustment> {
    let mut out: Vec<LegacyProbeAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_probe_adjustment(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_legacy_probe_adjustment(item: &LegacyProbeAdjustment) -> String {
    alert_alert_quota::adaptive_alert_variance_bucket(item.effective_exposure())
}
