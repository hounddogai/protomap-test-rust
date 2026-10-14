use super::*;

use std::collections::BTreeMap;

use crate::failover::agreement as failover_agreement;

pub fn summarize_fallback_failover_drafts_for_template(
    items: &[failover_agreement::FallbackFailoverDraft],
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

pub fn derive_fallback_health_check_templates_from_draft(
    items: &[failover_agreement::FallbackFailoverDraft],
) -> Vec<FallbackHealthCheckTemplate> {
    let mut out: Vec<FallbackHealthCheckTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_health_check_template(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_fallback_health_check_template(item: &FallbackHealthCheckTemplate) -> f64 {
    failover_agreement::fallback_failover_budget_weight(8, true)
}
