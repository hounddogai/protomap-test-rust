use super::*;

use std::collections::BTreeMap;

use crate::page::forecast as page_forecast;

pub fn summarize_rolling_page_metrics_for_variance(
    items: &[page_forecast::RollingPageMetric],
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

pub fn derive_lenient_page_variances_from_metric(
    items: &[page_forecast::RollingPageMetric],
) -> Vec<LenientPageVariance> {
    let mut out: Vec<LenientPageVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_page_variance(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_lenient_page_variance(item: &LenientPageVariance) -> String {
    page_forecast::classify_page_forecast(item.measured_value())
}
