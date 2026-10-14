use super::*;

use std::collections::BTreeMap;

use crate::zone::manual_zone_slot as zone_manual_zone_slot;

pub fn summarize_zone_templates_for_variance(items: &[zone_manual_zone_slot::ZoneTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_primary_health_check_variances_from_template(
    items: &[zone_manual_zone_slot::ZoneTemplate],
) -> Vec<PrimaryHealthCheckVariance> {
    let mut out: Vec<PrimaryHealthCheckVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_health_check_variance(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_primary_health_check_variance(item: &PrimaryHealthCheckVariance) -> f64 {
    zone_manual_zone_slot::zone_template_weight(3, true)
}
