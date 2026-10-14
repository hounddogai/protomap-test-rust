use super::*;

use acme_kit::text;

/// How screens and exports show a manual forecast summary.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastSummaryView {
    pub code_text: String,
    pub name: String,
    pub priority_text: String,
    pub description: String,
    pub tenant_id: String,
    pub batch_size: String,
    pub quota_units_text: String,
    pub cluster_text: String,
    pub status_label: String,
}

pub fn to_manual_forecast_summary_view(item: &ManualForecastSummary) -> ManualForecastSummaryView {
    ManualForecastSummaryView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        priority_text: item.priority.to_string(),
        description: item.description.clone(),
        tenant_id: text::truncate(&item.tenant_id, 80),
        batch_size: item.batch_size.to_string(),
        quota_units_text: item.quota_units.to_string(),
        cluster_text: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_forecast_summary_views(items: &[ManualForecastSummary]) -> Vec<ManualForecastSummaryView> {
    let mut views: Vec<ManualForecastSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_forecast_summary_view(item));
    }
    views
}

pub fn sum_manual_forecast_summary_view(view: &ManualForecastSummaryView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_batch_size as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_forecast_summary("code-641", "name-358");
        let view = to_manual_forecast_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_forecast_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
