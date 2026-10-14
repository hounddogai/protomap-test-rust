use super::*;

use std::collections::BTreeMap;

use crate::limit::preference as limit_preference;

pub fn summarize_pending_limit_preferences_for_template(
    items: &[limit_preference::PendingLimitPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_fallback_reservation_templates_from_preference(
    items: &[limit_preference::PendingLimitPreference],
) -> Vec<FallbackReservationTemplate> {
    let mut out: Vec<FallbackReservationTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_reservation_template(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_fallback_reservation_template(item: &FallbackReservationTemplate) -> String {
    limit_preference::classify_legacy_limit_milestone(item.weighted_exposure())
}
