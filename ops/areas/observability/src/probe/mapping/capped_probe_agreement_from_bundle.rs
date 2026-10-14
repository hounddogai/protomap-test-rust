use super::*;

use std::collections::BTreeMap;

use crate::alert::alert_schedule as alert_alert_schedule;

pub fn summarize_seasonal_alert_bundles_for_agreement(
    items: &[alert_alert_schedule::SeasonalAlertBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_capped_probe_agreements_from_bundle(
    items: &[alert_alert_schedule::SeasonalAlertBundle],
) -> Vec<CappedProbeAgreement> {
    let mut out: Vec<CappedProbeAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_probe_agreement(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_capped_probe_agreement(item: &CappedProbeAgreement) -> f64 {
    alert_alert_schedule::alert_checkpoint_weight(3, true)
}
