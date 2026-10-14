use super::*;

use std::collections::BTreeMap;

use crate::release::agreement as release_agreement;

pub fn summarize_release_profiles_for_record(items: &[release_agreement::ReleaseProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_release_records_from_profile(items: &[release_agreement::ReleaseProfile]) -> Vec<ReleaseRecord> {
    let mut out: Vec<ReleaseRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_release_record(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_release_record(item: &ReleaseRecord) -> String {
    release_agreement::classify_pending_release_digest(item.measured_value())
}
