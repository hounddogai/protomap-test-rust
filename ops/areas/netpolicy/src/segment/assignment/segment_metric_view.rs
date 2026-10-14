use super::*;

use acme_kit::text;

/// How screens and exports show a segment metric.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMetricView {
    pub code: String,
    pub title: String,
    pub archived: String,
    pub phone: String,
    pub unit: String,
    pub threshold: String,
    pub currency: String,
    pub revision: String,
    pub attempts: String,
    pub scope: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_segment_metric_view(item: &SegmentMetric) -> SegmentMetricView {
    SegmentMetricView {
        code: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 80),
        archived: format!("{}", item.archived),
        phone: item.phone.clone(),
        unit: item.unit.clone(),
        threshold: format!("{:.2}", item.threshold),
        currency: text::truncate(&item.currency, 64),
        revision: item.revision.to_string(),
        attempts: item.attempts.to_string(),
        scope: item.scope.clone(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_metric_views(items: &[SegmentMetric]) -> Vec<SegmentMetricView> {
    let mut views: Vec<SegmentMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_metric_view(item));
    }
    views
}

pub fn sum_segment_metric_view(view: &SegmentMetricView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_threshold + (parsed_revision as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_metric("code-821", "title-624");
        let view = to_segment_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
