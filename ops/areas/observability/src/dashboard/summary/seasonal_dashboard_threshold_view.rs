use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal dashboard threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardThresholdView {
    pub code_text: String,
    pub name: String,
    pub budget: String,
    pub grace_days: String,
    pub sequence: String,
    pub unit: String,
    pub capacity: String,
    pub country: String,
    pub starts_at: String,
    pub reviewer: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_seasonal_dashboard_threshold_view(item: &SeasonalDashboardThreshold) -> SeasonalDashboardThresholdView {
    SeasonalDashboardThresholdView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        budget: format!("{:.2}", item.budget),
        grace_days: item.grace_days.to_string(),
        sequence: item.sequence.to_string(),
        unit: item.unit.clone(),
        capacity: item.capacity.to_string(),
        country: item.country.clone(),
        starts_at: item.starts_at.to_string(),
        reviewer: item.reviewer.clone(),
        source: text::truncate(&item.source, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_dashboard_threshold_views(
    items: &[SeasonalDashboardThreshold],
) -> Vec<SeasonalDashboardThresholdView> {
    let mut views: Vec<SeasonalDashboardThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_dashboard_threshold_view(item));
    }
    views
}

pub fn sum_seasonal_dashboard_threshold_view(view: &SeasonalDashboardThresholdView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_budget + (parsed_grace_days as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_dashboard_threshold("code-532", "name-183");
        let view = to_seasonal_dashboard_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_dashboard_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
