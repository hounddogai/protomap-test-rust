use super::*;

use std::collections::BTreeMap;

use crate::egress::egress_agreement as egress_egress_agreement;

pub fn summarize_lenient_egress_agreements_for_cycle(
    items: &[egress_egress_agreement::LenientEgressAgreement],
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

pub fn derive_scheduled_egress_cycles_from_agreement(
    items: &[egress_egress_agreement::LenientEgressAgreement],
) -> Vec<ScheduledEgressCycle> {
    let mut out: Vec<ScheduledEgressCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_egress_cycle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_scheduled_egress_cycle(item: &ScheduledEgressCycle) -> String {
    egress_egress_agreement::egress_baseline_bucket(item.measured_value())
}
