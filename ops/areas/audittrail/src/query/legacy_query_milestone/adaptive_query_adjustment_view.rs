use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive query adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryAdjustmentView {
    pub code_text: String,
    pub title_text: String,
    pub minimum: String,
    pub contact_email: String,
    pub priority_text: String,
    pub status_label: String,
}

pub fn to_adaptive_query_adjustment_view(item: &AdaptiveQueryAdjustment) -> AdaptiveQueryAdjustmentView {
    AdaptiveQueryAdjustmentView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 40),
        minimum: format!("{:.2}", item.minimum),
        contact_email: item.contact_email.clone(),
        priority_text: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_query_adjustment_views(items: &[AdaptiveQueryAdjustment]) -> Vec<AdaptiveQueryAdjustmentView> {
    let mut views: Vec<AdaptiveQueryAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_query_adjustment_view(item));
    }
    views
}

pub fn sum_adaptive_query_adjustment_view(view: &AdaptiveQueryAdjustmentView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_minimum + (parsed_priority as f64))
}
