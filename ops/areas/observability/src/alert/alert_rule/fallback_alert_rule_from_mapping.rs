use super::*;

use std::collections::BTreeMap;

use crate::slo::adjustment as slo_adjustment;

pub fn summarize_slo_mappings_for_rule(items: &[slo_adjustment::SloMapping]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_fallback_alert_rules_from_mapping(items: &[slo_adjustment::SloMapping]) -> Vec<FallbackAlertRule> {
    let mut out: Vec<FallbackAlertRule> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_alert_rule(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_fallback_alert_rule(item: &FallbackAlertRule) -> String {
    slo_adjustment::default_slo_adjustment_bucket(item.weighted_exposure())
}
