use super::*;

use std::collections::BTreeMap;

use crate::region_pin::adjustment as region_pin_adjustment;

pub fn summarize_region_pin_agreements_for_forecast(
    items: &[region_pin_adjustment::RegionPinAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_plan_forecasts_from_agreement(items: &[region_pin_adjustment::RegionPinAgreement]) -> Vec<PlanForecast> {
    let mut out: Vec<PlanForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_forecast(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_plan_forecast(item: &PlanForecast) -> String {
    region_pin_adjustment::normalize_capped_region_pin_manifest_code(&item.code)
}
