use super::*;

use std::collections::BTreeMap;

use crate::forecast::template as forecast_template;

pub fn summarize_regional_forecast_rules_for_variance(
    items: &[forecast_template::RegionalForecastRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_primary_reservation_variances_from_rule(
    items: &[forecast_template::RegionalForecastRule],
) -> Vec<PrimaryReservationVariance> {
    let mut out: Vec<PrimaryReservationVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_reservation_variance(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_primary_reservation_variance(item: &PrimaryReservationVariance) -> String {
    forecast_template::strict_forecast_allocation_bucket(item.weighted_volume())
}
