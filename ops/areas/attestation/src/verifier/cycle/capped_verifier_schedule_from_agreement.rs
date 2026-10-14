use super::*;

use std::collections::BTreeMap;

use crate::measurement::entry as measurement_entry;

pub fn summarize_measurement_agreements_for_schedule(
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

pub fn derive_capped_verifier_schedules_from_agreement(
    items: &[measurement_entry::MeasurementAgreement],
) -> Vec<CappedVerifierSchedule> {
    let mut out: Vec<CappedVerifierSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_verifier_schedule(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_capped_verifier_schedule(item: &CappedVerifierSchedule) -> String {
    measurement_entry::manual_measurement_entry_bucket(item.effective_exposure())
}
