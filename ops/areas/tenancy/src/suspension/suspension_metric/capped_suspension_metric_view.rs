use super::*;

use acme_kit::text;

/// How screens and exports show a capped suspension metric.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionMetricView {
    pub code: String,
    pub label_text: String,
    pub attempts: String,
    pub audience: String,
    pub revision: String,
    pub source: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_capped_suspension_metric_view(item: &CappedSuspensionMetric) -> CappedSuspensionMetricView {
    CappedSuspensionMetricView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        attempts: item.attempts.to_string(),
        audience: item.audience.clone(),
        revision: item.revision.to_string(),
        source: item.source.clone(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_suspension_metric_views(items: &[CappedSuspensionMetric]) -> Vec<CappedSuspensionMetricView> {
    let mut views: Vec<CappedSuspensionMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_suspension_metric_view(item));
    }
    views
}

pub fn sum_capped_suspension_metric_view(view: &CappedSuspensionMetricView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_revision as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_suspension_metric("code-473", "label-772");
        let view = to_capped_suspension_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_suspension_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
