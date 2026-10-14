use super::*;

/// How screens and exports show a adaptive region pin metric.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinMetricView {
    pub code: String,
    pub title: String,
    pub score: String,
    pub source: String,
    pub scope: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_adaptive_region_pin_metric_view(item: &AdaptiveRegionPinMetric) -> AdaptiveRegionPinMetricView {
    AdaptiveRegionPinMetricView {
        code: item.code.clone(),
        title: item.title.clone(),
        score: format!("{:.2}", item.score),
        source: item.source.clone(),
        scope: item.scope.clone(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_region_pin_metric_views(items: &[AdaptiveRegionPinMetric]) -> Vec<AdaptiveRegionPinMetricView> {
    let mut views: Vec<AdaptiveRegionPinMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_region_pin_metric_view(item));
    }
    views
}

pub fn sum_adaptive_region_pin_metric_view(view: &AdaptiveRegionPinMetricView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_score + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_region_pin_metric("code-981", "title-743");
        let view = to_adaptive_region_pin_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_region_pin_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
