use super::*;

use std::collections::BTreeMap;

use crate::peering::ledger as peering_ledger;

pub fn summarize_global_peering_ledgers_for_forecast(
    items: &[peering_ledger::GlobalPeeringLedger],
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

pub fn derive_tiered_segment_forecasts_from_ledger(
    items: &[peering_ledger::GlobalPeeringLedger],
) -> Vec<TieredSegmentForecast> {
    let mut out: Vec<TieredSegmentForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_segment_forecast(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_tiered_segment_forecast(item: &TieredSegmentForecast) -> f64 {
    peering_ledger::regional_peering_quota_weight(1, true)
}
