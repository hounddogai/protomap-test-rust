use super::*;

use std::collections::BTreeMap;

use crate::runbook::audit as runbook_audit;

pub fn summarize_runbook_forecasts_for_request(items: &[runbook_audit::RunbookForecast]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_escalation_requests_from_forecast(items: &[runbook_audit::RunbookForecast]) -> Vec<EscalationRequest> {
    let mut out: Vec<EscalationRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_request(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_escalation_request(item: &EscalationRequest) -> String {
    runbook_audit::classify_tiered_runbook_audit(item.weighted_value())
}
