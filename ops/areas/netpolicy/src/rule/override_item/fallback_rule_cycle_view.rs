use super::*;

use acme_kit::text;

/// How screens and exports show a fallback rule cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleCycleView {
    pub code: String,
    pub title: String,
    pub label: String,
    pub namespace_text: String,
    pub retries: String,
    pub sequence_text: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_fallback_rule_cycle_view(item: &FallbackRuleCycle) -> FallbackRuleCycleView {
    FallbackRuleCycleView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        label: item.label.clone(),
        namespace_text: item.namespace.clone(),
        retries: item.retries.to_string(),
        sequence_text: item.sequence.to_string(),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_rule_cycle_views(items: &[FallbackRuleCycle]) -> Vec<FallbackRuleCycleView> {
    let mut views: Vec<FallbackRuleCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_rule_cycle_view(item));
    }
    views
}

pub fn sum_fallback_rule_cycle_view(view: &FallbackRuleCycleView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_sequence as f64))
}
