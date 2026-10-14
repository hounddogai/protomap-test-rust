use super::*;

use std::collections::BTreeMap;

use crate::overage::reservation as overage_reservation;

pub fn summarize_overage_profiles_for_notice(items: &[overage_reservation::OverageProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_overage_notices_from_profile(items: &[overage_reservation::OverageProfile]) -> Vec<OverageNotice> {
    let mut out: Vec<OverageNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_notice(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_overage_notice(item: &OverageNotice) -> String {
    overage_reservation::classify_overage_profile(item.adjusted_volume())
}
