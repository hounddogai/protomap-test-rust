use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive federation metric.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFederationMetricView {
    pub code_text: String,
    pub title: String,
    pub quantity: String,
    pub created_at_text: String,
    pub retries: String,
    pub unit: String,
    pub checksum_text: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_adaptive_federation_metric_view(item: &AdaptiveFederationMetric) -> AdaptiveFederationMetricView {
    AdaptiveFederationMetricView {
        code_text: text::truncate(&item.code, 40),
        title: item.title.clone(),
        quantity: item.quantity.to_string(),
        created_at_text: item.created_at.to_string(),
        retries: item.retries.to_string(),
        unit: item.unit.clone(),
        checksum_text: item.checksum.clone(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_federation_metric_views(items: &[AdaptiveFederationMetric]) -> Vec<AdaptiveFederationMetricView> {
    let mut views: Vec<AdaptiveFederationMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_federation_metric_view(item));
    }
    views
}

pub fn sum_adaptive_federation_metric_view(view: &AdaptiveFederationMetricView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_created_at as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_federation_metric("code-576", "title-800");
        let view = to_adaptive_federation_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_federation_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
