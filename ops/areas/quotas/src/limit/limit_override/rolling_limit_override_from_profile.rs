use super::*;

use std::collections::BTreeMap;

use crate::reservation::window as reservation_window;

pub fn summarize_seasonal_reservation_profiles_for_override(
    items: &[reservation_window::SeasonalReservationProfile],
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

pub fn derive_rolling_limit_overrides_from_profile(
    items: &[reservation_window::SeasonalReservationProfile],
) -> Vec<RollingLimitOverride> {
    let mut out: Vec<RollingLimitOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_limit_override(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_rolling_limit_override(item: &RollingLimitOverride) -> f64 {
    reservation_window::seasonal_reservation_profile_weight(9, true)
}
