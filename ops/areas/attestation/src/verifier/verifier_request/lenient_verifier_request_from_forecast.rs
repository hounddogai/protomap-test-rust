use super::*;

use std::collections::BTreeMap;

use crate::quote::forecast as quote_forecast;

pub fn summarize_quote_forecasts_for_request(items: &[quote_forecast::QuoteForecast]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_lenient_verifier_requests_from_forecast(
    items: &[quote_forecast::QuoteForecast],
) -> Vec<LenientVerifierRequest> {
    let mut out: Vec<LenientVerifierRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_verifier_request(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_lenient_verifier_request(item: &LenientVerifierRequest) -> f64 {
    quote_forecast::quote_forecast_weight(3, true)
}
