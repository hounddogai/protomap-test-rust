use super::*;

use acme_kit::text;

/// How screens and exports show a alert metric.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertMetricView {
    pub code: String,
    pub label_text: String,
    pub version_text: String,
    pub country: String,
    pub scope: String,
    pub floor_value: String,
    pub sequence_text: String,
    pub retries: String,
    pub updated_at_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_alert_metric_view(item: &AlertMetric) -> AlertMetricView {
    AlertMetricView {
        code: text::truncate(&item.code, 80),
        label_text: text::truncate(&item.label, 64),
        version_text: item.version.to_string(),
        country: item.country.clone(),
        scope: item.scope.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        sequence_text: item.sequence.to_string(),
        retries: item.retries.to_string(),
        updated_at_text: item.updated_at.to_string(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_metric_views(items: &[AlertMetric]) -> Vec<AlertMetricView> {
    let mut views: Vec<AlertMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_metric_view(item));
    }
    views
}

pub fn sum_alert_metric_view(view: &AlertMetricView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_version as f64 + parsed_floor_value + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_alert_metric("code-651", "label-600");
        let view = to_alert_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_alert_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
