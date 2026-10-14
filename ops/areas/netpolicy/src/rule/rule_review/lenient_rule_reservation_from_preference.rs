use super::*;

use std::collections::BTreeMap;

use crate::segment::forecast as segment_forecast;

pub fn summarize_pending_segment_preferences_for_reservation(
    items: &[segment_forecast::PendingSegmentPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_lenient_rule_reservations_from_preference(
    items: &[segment_forecast::PendingSegmentPreference],
) -> Vec<LenientRuleReservation> {
    let mut out: Vec<LenientRuleReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_rule_reservation(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_lenient_rule_reservation(item: &LenientRuleReservation) -> String {
    segment_forecast::classify_tiered_segment_forecast(item.effective_exposure())
}
