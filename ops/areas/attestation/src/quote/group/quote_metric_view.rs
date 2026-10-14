use super::*;

/// How screens and exports show a quote metric.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMetricView {
    pub code_text: String,
    pub title_text: String,
    pub scope: String,
    pub pinned: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_quote_metric_view(item: &QuoteMetric) -> QuoteMetricView {
    QuoteMetricView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        scope: item.scope.clone(),
        pinned: format!("{}", item.pinned),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_metric_views(items: &[QuoteMetric]) -> Vec<QuoteMetricView> {
    let mut views: Vec<QuoteMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_metric_view(item));
    }
    views
}

pub fn sum_quote_metric_view(view: &QuoteMetricView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}
