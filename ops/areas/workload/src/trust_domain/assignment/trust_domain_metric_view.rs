use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain metric.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainMetricView {
    pub code: String,
    pub label: String,
    pub ratio: String,
    pub summary: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_trust_domain_metric_view(item: &TrustDomainMetric) -> TrustDomainMetricView {
    TrustDomainMetricView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        ratio: format!("{:.2}", item.ratio),
        summary: item.summary.clone(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_metric_views(items: &[TrustDomainMetric]) -> Vec<TrustDomainMetricView> {
    let mut views: Vec<TrustDomainMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_metric_view(item));
    }
    views
}

pub fn sum_trust_domain_metric_view(view: &TrustDomainMetricView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_ratio + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_metric("code-357", "label-572");
        let view = to_trust_domain_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
