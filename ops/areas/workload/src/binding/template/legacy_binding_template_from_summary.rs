use super::*;

use std::collections::BTreeMap;

use crate::token::tier as token_tier;

pub fn summarize_lenient_token_summaries_for_template(
    items: &[token_tier::LenientTokenSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_legacy_binding_templates_from_summary(
    items: &[token_tier::LenientTokenSummary],
) -> Vec<LegacyBindingTemplate> {
    let mut out: Vec<LegacyBindingTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_binding_template(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_legacy_binding_template(item: &LegacyBindingTemplate) -> f64 {
    token_tier::lenient_token_summary_weight(7, true)
}
