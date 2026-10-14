use super::*;

use std::collections::BTreeMap;

use crate::measurement::entry as measurement_entry;

pub fn summarize_measurement_agreements_for_plan(
    items: &[measurement_entry::MeasurementAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_manual_quote_plans_from_agreement(
    items: &[measurement_entry::MeasurementAgreement],
) -> Vec<ManualQuotePlan> {
    let mut out: Vec<ManualQuotePlan> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_quote_plan(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_manual_quote_plan(item: &ManualQuotePlan) -> String {
    measurement_entry::normalize_measurement_agreement_code(&item.code)
}
