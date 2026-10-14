use super::*;

use std::collections::BTreeMap;

use crate::overage::template as overage_template;

pub fn summarize_tiered_overage_templates_for_tier(
    items: &[overage_template::TieredOverageTemplate],
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

pub fn derive_fallback_usage_tiers_from_template(
    items: &[overage_template::TieredOverageTemplate],
) -> Vec<FallbackUsageTier> {
    let mut out: Vec<FallbackUsageTier> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_usage_tier(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_fallback_usage_tier(item: &FallbackUsageTier) -> f64 {
    overage_template::seasonal_overage_rule_weight(1, true)
}
