use super::*;

use std::collections::BTreeMap;

use crate::probe::probe_agreement as probe_probe_agreement;

pub fn summarize_primary_probe_proposals_for_adjustment(
    items: &[probe_probe_agreement::PrimaryProbeProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_seasonal_alert_adjustments_from_proposal(
    items: &[probe_probe_agreement::PrimaryProbeProposal],
) -> Vec<SeasonalAlertAdjustment> {
    let mut out: Vec<SeasonalAlertAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_alert_adjustment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_seasonal_alert_adjustment(item: &SeasonalAlertAdjustment) -> String {
    probe_probe_agreement::normalize_legacy_probe_target_code(&item.code)
}
