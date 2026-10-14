use super::*;

use std::collections::BTreeMap;

use crate::silence::metric as silence_metric;

pub fn summarize_regional_silence_overrides_for_audit(
    items: &[silence_metric::RegionalSilenceOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_legacy_slo_audits_from_override(
    items: &[silence_metric::RegionalSilenceOverride],
) -> Vec<LegacySloAudit> {
    let mut out: Vec<LegacySloAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_slo_audit(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_legacy_slo_audit(item: &LegacySloAudit) -> String {
    silence_metric::normalize_manual_silence_cycle_code(&item.code)
}
