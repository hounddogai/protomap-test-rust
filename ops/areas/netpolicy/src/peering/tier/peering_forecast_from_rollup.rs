use super::*;

use std::collections::BTreeMap;

use crate::egress::cycle as egress_cycle;

pub fn summarize_legacy_egress_rollups_for_forecast(
    items: &[egress_cycle::LegacyEgressRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_peering_forecasts_from_rollup(items: &[egress_cycle::LegacyEgressRollup]) -> Vec<PeeringForecast> {
    let mut out: Vec<PeeringForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_forecast(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_peering_forecast(item: &PeeringForecast) -> String {
    egress_cycle::classify_egress_notice(item.projected_total())
}
