use super::*;

use acme_kit::text;

/// How screens and exports show a manual failover metric.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFailoverMetricView {
    pub code_text: String,
    pub title: String,
    pub visible_text: String,
    pub timezone: String,
    pub batch_size: String,
    pub principal_text: String,
    pub description: String,
    pub active_text: String,
    pub priority: String,
    pub checksum: String,
    pub source_text: String,
    pub status_label: String,
}

pub fn to_manual_failover_metric_view(item: &ManualFailoverMetric) -> ManualFailoverMetricView {
    ManualFailoverMetricView {
        code_text: text::truncate(&item.code, 40),
        title: item.title.clone(),
        visible_text: format!("{}", item.visible),
        timezone: text::truncate(&item.timezone, 120),
        batch_size: item.batch_size.to_string(),
        principal_text: item.principal.clone(),
        description: item.description.clone(),
        active_text: format!("{}", item.active),
        priority: item.priority.to_string(),
        checksum: item.checksum.clone(),
        source_text: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_failover_metric_views(items: &[ManualFailoverMetric]) -> Vec<ManualFailoverMetricView> {
    let mut views: Vec<ManualFailoverMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_failover_metric_view(item));
    }
    views
}

pub fn sum_manual_failover_metric_view(view: &ManualFailoverMetricView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_failover_metric("code-42", "title-57");
        let view = to_manual_failover_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_failover_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
