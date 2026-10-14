use super::*;

use std::collections::BTreeMap;

use crate::zone::policy as zone_policy;

pub fn summarize_zone_policies_for_tier(items: &[zone_policy::ZonePolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_dns_record_tiers_from_policy(items: &[zone_policy::ZonePolicy]) -> Vec<DnsRecordTier> {
    let mut out: Vec<DnsRecordTier> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_tier(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_dns_record_tier(item: &DnsRecordTier) -> String {
    zone_policy::classify_zone_threshold(item.measured_value())
}
