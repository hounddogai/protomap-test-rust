use super::*;

use std::collections::BTreeMap;

use crate::data_export::plan as data_export_plan;

pub fn summarize_data_export_reviews_for_ledger(items: &[data_export_plan::DataExportReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_sink_ledgers_from_review(items: &[data_export_plan::DataExportReview]) -> Vec<SinkLedger> {
    let mut out: Vec<SinkLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_ledger(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_sink_ledger(item: &SinkLedger) -> String {
    data_export_plan::classify_data_export_plan(item.adjusted_exposure())
}
