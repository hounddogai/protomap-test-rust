use super::*;

use std::collections::BTreeMap;

use crate::health_check::rolling_check_milestone as health_check_rolling_check_milestone;

pub fn summarize_health_check_tiers_for_digest(
    items: &[health_check_rolling_check_milestone::HealthCheckTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_zone_digests_from_tier(
    items: &[health_check_rolling_check_milestone::HealthCheckTier],
) -> Vec<ZoneDigest> {
    let mut out: Vec<ZoneDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_digest(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_zone_digest(item: &ZoneDigest) -> String {
    health_check_rolling_check_milestone::normalize_tiered_health_check_audit_code(&item.code)
}
