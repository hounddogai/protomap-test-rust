use super::*;

use std::collections::BTreeMap;

use crate::page::template as page_template;

pub fn summarize_rolling_page_templates_for_summary(
    items: &[page_template::RollingPageTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_timeline_summaries_from_template(items: &[page_template::RollingPageTemplate]) -> Vec<TimelineSummary> {
    let mut out: Vec<TimelineSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_summary(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_timeline_summary(item: &TimelineSummary) -> String {
    page_template::scheduled_page_target_bucket(item.effective_value())
}
