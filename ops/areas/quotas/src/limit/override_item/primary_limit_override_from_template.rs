use super::*;

use std::collections::BTreeMap;

use crate::reservation::exception as reservation_exception;

pub fn summarize_primary_reservation_templates_for_override(
    items: &[reservation_exception::PrimaryReservationTemplate],
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

pub fn derive_primary_limit_overrides_from_template(
    items: &[reservation_exception::PrimaryReservationTemplate],
) -> Vec<PrimaryLimitOverride> {
    let mut out: Vec<PrimaryLimitOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_limit_override(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_primary_limit_override(item: &PrimaryLimitOverride) -> String {
    reservation_exception::classify_reservation_mapping(item.weighted_volume())
}
