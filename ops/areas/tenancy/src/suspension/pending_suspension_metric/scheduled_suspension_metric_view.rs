use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled suspension metric.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionMetricView {
    pub code: String,
    pub label: String,
    pub version: String,
    pub approved_text: String,
    pub retries_text: String,
    pub status_label: String,
}

pub fn to_scheduled_suspension_metric_view(item: &ScheduledSuspensionMetric) -> ScheduledSuspensionMetricView {
    ScheduledSuspensionMetricView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        version: item.version.to_string(),
        approved_text: format!("{}", item.approved),
        retries_text: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_suspension_metric_views(items: &[ScheduledSuspensionMetric]) -> Vec<ScheduledSuspensionMetricView> {
    let mut views: Vec<ScheduledSuspensionMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_suspension_metric_view(item));
    }
    views
}

pub fn sum_scheduled_suspension_metric_view(view: &ScheduledSuspensionMetricView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_version as f64 + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_suspension_metric("code-816", "label-207");
        let view = to_scheduled_suspension_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_suspension_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
