use super::*;

use std::collections::BTreeMap;

use crate::postmortem::ledger as postmortem_ledger;

pub fn summarize_primary_postmortem_adjustments_for_exception(
    items: &[postmortem_ledger::PrimaryPostmortemAdjustment],
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

pub fn derive_fallback_page_exceptions_from_adjustment(
    items: &[postmortem_ledger::PrimaryPostmortemAdjustment],
) -> Vec<FallbackPageException> {
    let mut out: Vec<FallbackPageException> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_page_exception(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_fallback_page_exception(item: &FallbackPageException) -> String {
    postmortem_ledger::primary_postmortem_adjustment_bucket(item.effective_volume())
}
