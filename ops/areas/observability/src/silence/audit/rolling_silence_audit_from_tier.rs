use super::*;

use std::collections::BTreeMap;

use crate::dashboard::tier as dashboard_tier;

pub fn summarize_capped_dashboard_tiers_for_audit(
    items: &[dashboard_tier::CappedDashboardTier],
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

pub fn derive_rolling_silence_audits_from_tier(
    items: &[dashboard_tier::CappedDashboardTier],
) -> Vec<RollingSilenceAudit> {
    let mut out: Vec<RollingSilenceAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_silence_audit(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_rolling_silence_audit(item: &RollingSilenceAudit) -> String {
    dashboard_tier::normalize_dashboard_agreement_code(&item.code)
}
