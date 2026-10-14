use super::*;

use std::collections::BTreeMap;

use crate::zone::zone_preference as zone_zone_preference;

pub fn summarize_zone_drafts_for_record(items: &[zone_zone_preference::ZoneDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_adaptive_delegation_records_from_draft(
    items: &[zone_zone_preference::ZoneDraft],
) -> Vec<AdaptiveDelegationRecord> {
    let mut out: Vec<AdaptiveDelegationRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_delegation_record(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_adaptive_delegation_record(item: &AdaptiveDelegationRecord) -> String {
    zone_zone_preference::lenient_zone_preference_bucket(item.effective_load())
}
