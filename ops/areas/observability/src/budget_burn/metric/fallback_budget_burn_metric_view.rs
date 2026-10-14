use super::*;

use acme_kit::text;

/// How screens and exports show a fallback budget burn metric.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnMetricView {
    pub code: String,
    pub name_text: String,
    pub burst_text: String,
    pub quota_units: String,
    pub channel: String,
    pub pinned_text: String,
    pub notes: String,
    pub country: String,
    pub visible: String,
    pub checksum: String,
    pub cluster_text: String,
    pub status_label: String,
}

pub fn to_fallback_budget_burn_metric_view(item: &FallbackBudgetBurnMetric) -> FallbackBudgetBurnMetricView {
    FallbackBudgetBurnMetricView {
        code: text::truncate(&item.code, 64),
        name_text: item.name.clone(),
        burst_text: item.burst.to_string(),
        quota_units: item.quota_units.to_string(),
        channel: item.channel.clone(),
        pinned_text: format!("{}", item.pinned),
        notes: text::truncate(&item.notes, 120),
        country: item.country.clone(),
        visible: format!("{}", item.visible),
        checksum: item.checksum.clone(),
        cluster_text: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_budget_burn_metric_views(items: &[FallbackBudgetBurnMetric]) -> Vec<FallbackBudgetBurnMetricView> {
    let mut views: Vec<FallbackBudgetBurnMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_budget_burn_metric_view(item));
    }
    views
}

pub fn sum_fallback_budget_burn_metric_view(view: &FallbackBudgetBurnMetricView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_budget_burn_metric("code-362", "name-977");
        let view = to_fallback_budget_burn_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_budget_burn_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
