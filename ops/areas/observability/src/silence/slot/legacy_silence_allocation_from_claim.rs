use super::*;

use std::collections::BTreeMap;

use crate::alert::score as alert_score;

pub fn summarize_regional_alert_claims_for_allocation(
    items: &[alert_score::RegionalAlertClaim],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_legacy_silence_allocations_from_claim(
    items: &[alert_score::RegionalAlertClaim],
) -> Vec<LegacySilenceAllocation> {
    let mut out: Vec<LegacySilenceAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_silence_allocation(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_legacy_silence_allocation_extra(item: &LegacySilenceAllocation) -> String {
    alert_score::regional_alert_claim_bucket(item.projected_volume())
}
