use super::*;

use std::collections::BTreeMap;

use crate::slo::entry as slo_entry;

pub fn summarize_slo_notices_for_profile(items: &[slo_entry::SloNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_primary_silence_profiles_from_notice(items: &[slo_entry::SloNotice]) -> Vec<PrimarySilenceProfile> {
    let mut out: Vec<PrimarySilenceProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_silence_profile(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_primary_silence_profile(item: &PrimarySilenceProfile) -> String {
    slo_entry::normalize_scheduled_slo_reservation_code(&item.code)
}
