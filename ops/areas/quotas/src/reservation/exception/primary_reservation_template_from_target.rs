use super::*;

use std::collections::BTreeMap;

use crate::limit::mapping as limit_mapping;

pub fn summarize_limit_targets_for_template(items: &[limit_mapping::LimitTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_primary_reservation_templates_from_target(
    items: &[limit_mapping::LimitTarget],
) -> Vec<PrimaryReservationTemplate> {
    let mut out: Vec<PrimaryReservationTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_reservation_template(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_primary_reservation_template(item: &PrimaryReservationTemplate) -> f64 {
    limit_mapping::limit_target_weight(5, true)
}
