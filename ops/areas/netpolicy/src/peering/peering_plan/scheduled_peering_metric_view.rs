use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled peering metric.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPeeringMetricView {
    pub code: String,
    pub name: String,
    pub approved: String,
    pub summary: String,
    pub description: String,
    pub segment: String,
    pub priority_text: String,
    pub currency: String,
    pub maximum: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_scheduled_peering_metric_view(item: &ScheduledPeeringMetric) -> ScheduledPeeringMetricView {
    ScheduledPeeringMetricView {
        code: item.code.clone(),
        name: item.name.clone(),
        approved: format!("{}", item.approved),
        summary: item.summary.clone(),
        description: text::truncate(&item.description, 64),
        segment: item.segment.clone(),
        priority_text: item.priority.to_string(),
        currency: item.currency.clone(),
        maximum: format!("{:.2}", item.maximum),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_peering_metric_views(items: &[ScheduledPeeringMetric]) -> Vec<ScheduledPeeringMetricView> {
    let mut views: Vec<ScheduledPeeringMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_peering_metric_view(item));
    }
    views
}

pub fn sum_scheduled_peering_metric_view(view: &ScheduledPeeringMetricView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_priority as f64 + parsed_maximum + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_peering_metric("code-832", "name-917");
        let view = to_scheduled_peering_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_peering_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
