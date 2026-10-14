use super::*;

use std::collections::BTreeMap;

use crate::slo::slo_budget as slo_slo_budget;

pub fn summarize_manual_slo_templates_for_schedule(
    items: &[slo_slo_budget::ManualSloTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_default_probe_schedules_from_template(
    items: &[slo_slo_budget::ManualSloTemplate],
) -> Vec<DefaultProbeSchedule> {
    let mut out: Vec<DefaultProbeSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_default_probe_schedule(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_default_probe_schedule(item: &DefaultProbeSchedule) -> String {
    slo_slo_budget::manual_slo_template_bucket(item.net_exposure())
}
