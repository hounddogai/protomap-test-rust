use super::*;

/// How screens and exports show a default dashboard metric.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardMetricView {
    pub code_text: String,
    pub label: String,
    pub duration_minutes_text: String,
    pub rate: String,
    pub channel: String,
    pub external_ref: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_default_dashboard_metric_view(item: &DefaultDashboardMetric) -> DefaultDashboardMetricView {
    DefaultDashboardMetricView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        rate: format!("{:.2}", item.rate),
        channel: item.channel.clone(),
        external_ref: item.external_ref.clone(),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_dashboard_metric_views(items: &[DefaultDashboardMetric]) -> Vec<DefaultDashboardMetricView> {
    let mut views: Vec<DefaultDashboardMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_dashboard_metric_view(item));
    }
    views
}

pub fn sum_default_dashboard_metric_view(view: &DefaultDashboardMetricView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_dashboard_metric("code-632", "label-233");
        let view = to_default_dashboard_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_dashboard_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
