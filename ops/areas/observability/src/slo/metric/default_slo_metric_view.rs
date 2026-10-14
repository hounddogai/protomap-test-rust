use super::*;

use acme_kit::text;

/// How screens and exports show a default slo metric.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSloMetricView {
    pub code: String,
    pub title_text: String,
    pub checksum_text: String,
    pub ceiling_text: String,
    pub minimum: String,
    pub archived: String,
    pub weight_text: String,
    pub status_label: String,
}

pub fn to_default_slo_metric_view(item: &DefaultSloMetric) -> DefaultSloMetricView {
    DefaultSloMetricView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 40),
        checksum_text: item.checksum.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        minimum: format!("{:.2}", item.minimum),
        archived: format!("{}", item.archived),
        weight_text: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_slo_metric_views(items: &[DefaultSloMetric]) -> Vec<DefaultSloMetricView> {
    let mut views: Vec<DefaultSloMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_slo_metric_view(item));
    }
    views
}

pub fn sum_default_slo_metric_view(view: &DefaultSloMetricView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_ceiling + parsed_minimum + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_slo_metric("code-392", "title-870");
        let view = to_default_slo_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_slo_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
