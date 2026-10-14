use super::*;

use std::collections::BTreeMap;

use crate::identity::identity_schedule as identity_identity_schedule;

pub fn summarize_seasonal_identity_schedules_for_override(
    items: &[identity_identity_schedule::SeasonalIdentitySchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_legacy_federation_overrides_from_schedule(
    items: &[identity_identity_schedule::SeasonalIdentitySchedule],
) -> Vec<LegacyFederationOverride> {
    let mut out: Vec<LegacyFederationOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_federation_override(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_legacy_federation_override(item: &LegacyFederationOverride) -> f64 {
    identity_identity_schedule::seasonal_identity_schedule_weight(6, true)
}
