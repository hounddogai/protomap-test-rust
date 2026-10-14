use super::*;

use std::collections::BTreeMap;

use crate::dns_record::milestone as dns_record_milestone;

pub fn summarize_dns_record_milestones_for_preference(
    items: &[dns_record_milestone::DnsRecordMilestone],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_zone_preferences_from_milestone(
    items: &[dns_record_milestone::DnsRecordMilestone],
) -> Vec<ZonePreference> {
    let mut out: Vec<ZonePreference> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_preference(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_zone_preference(item: &ZonePreference) -> String {
    dns_record_milestone::normalize_rolling_dns_record_item_code(&item.code)
}
