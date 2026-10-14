use super::*;

use std::collections::BTreeMap;

use crate::quote::targets as quote_targets;

pub fn summarize_scheduled_quote_groups_for_schedule(
    items: &[quote_targets::ScheduledQuoteGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_fallback_quote_schedules_from_group(
    items: &[quote_targets::ScheduledQuoteGroup],
) -> Vec<FallbackQuoteSchedule> {
    let mut out: Vec<FallbackQuoteSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_quote_schedule(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_fallback_quote_schedule(item: &FallbackQuoteSchedule) -> f64 {
    quote_targets::regional_quote_metric_weight(6, true)
}
