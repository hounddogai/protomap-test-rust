use super::*;

use acme_kit::text;

/// How screens and exports show a credential metric.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialMetricView {
    pub code: String,
    pub name: String,
    pub minimum_text: String,
    pub threshold_text: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_credential_metric_view(item: &CredentialMetric) -> CredentialMetricView {
    CredentialMetricView {
        code: item.code.clone(),
        name: item.name.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        threshold_text: format!("{:.2}", item.threshold),
        host: text::truncate(&item.host, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_metric_views(items: &[CredentialMetric]) -> Vec<CredentialMetricView> {
    let mut views: Vec<CredentialMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_metric_view(item));
    }
    views
}

pub fn sum_credential_metric_view(view: &CredentialMetricView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_minimum + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_metric("code-3", "name-633");
        let view = to_credential_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
