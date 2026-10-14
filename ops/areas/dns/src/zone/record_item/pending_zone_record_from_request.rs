use super::*;

use std::collections::BTreeMap;

use crate::zone::request as zone_request;

pub fn summarize_zone_requests_for_record(items: &[zone_request::ZoneRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_pending_zone_records_from_request(items: &[zone_request::ZoneRequest]) -> Vec<PendingZoneRecord> {
    let mut out: Vec<PendingZoneRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_zone_record(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_pending_zone_record(item: &PendingZoneRecord) -> String {
    zone_request::normalize_zone_milestone_code(&item.code)
}
