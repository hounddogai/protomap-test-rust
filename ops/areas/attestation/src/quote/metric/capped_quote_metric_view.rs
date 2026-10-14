use super::*;

use acme_kit::text;

/// How screens and exports show a capped quote metric.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedQuoteMetricView {
    pub code: String,
    pub title_text: String,
    pub description: String,
    pub weight_text: String,
    pub currency_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_capped_quote_metric_view(item: &CappedQuoteMetric) -> CappedQuoteMetricView {
    CappedQuoteMetricView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        description: item.description.clone(),
        weight_text: format!("{:.2}", item.weight),
        currency_text: item.currency.clone(),
        namespace: text::truncate(&item.namespace, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_quote_metric_views(items: &[CappedQuoteMetric]) -> Vec<CappedQuoteMetricView> {
    let mut views: Vec<CappedQuoteMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_quote_metric_view(item));
    }
    views
}

pub fn sum_capped_quote_metric_view(view: &CappedQuoteMetricView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_quote_metric("code-795", "title-489");
        let view = to_capped_quote_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_quote_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
