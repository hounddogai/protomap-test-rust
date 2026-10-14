use super::*;

use acme_kit::text;

/// How screens and exports show a fallback failover metric.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverMetricView {
    pub code: String,
    pub name: String,
    pub ratio_text: String,
    pub segment_text: String,
    pub rate: String,
    pub currency: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_fallback_failover_metric_view(item: &FallbackFailoverMetric) -> FallbackFailoverMetricView {
    FallbackFailoverMetricView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        ratio_text: format!("{:.2}", item.ratio),
        segment_text: text::truncate(&item.segment, 80),
        rate: format!("{:.2}", item.rate),
        currency: item.currency.clone(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_failover_metric_views(items: &[FallbackFailoverMetric]) -> Vec<FallbackFailoverMetricView> {
    let mut views: Vec<FallbackFailoverMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_failover_metric_view(item));
    }
    views
}

pub fn sum_fallback_failover_metric_view(view: &FallbackFailoverMetricView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_ratio + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_failover_metric("code-708", "name-738");
        let view = to_fallback_failover_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_failover_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
