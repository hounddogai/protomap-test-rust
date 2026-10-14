use super::*;

use std::collections::BTreeMap;

use crate::runbook::entry as runbook_entry;

pub fn summarize_tiered_runbook_templates_for_cycle(
    items: &[runbook_entry::TieredRunbookTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_fallback_escalation_cycles_from_template(
    items: &[runbook_entry::TieredRunbookTemplate],
) -> Vec<FallbackEscalationCycle> {
    let mut out: Vec<FallbackEscalationCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_escalation_cycle(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_fallback_escalation_cycle(item: &FallbackEscalationCycle) -> String {
    runbook_entry::runbook_entry_bucket(item.effective_volume())
}
