use super::*;

use std::collections::BTreeMap;

use crate::slo::slo_audit as slo_slo_audit;

pub fn summarize_lenient_slo_audits_for_cycle(items: &[slo_slo_audit::LenientSloAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_tiered_silence_cycles_from_audit(items: &[slo_slo_audit::LenientSloAudit]) -> Vec<TieredSilenceCycle> {
    let mut out: Vec<TieredSilenceCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_silence_cycle(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_tiered_silence_cycle(item: &TieredSilenceCycle) -> String {
    slo_slo_audit::lenient_slo_audit_bucket(item.projected_load())
}
