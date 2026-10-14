use super::*;

use acme_kit::text;

/// How screens and exports show a tiered forecast milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastMilestoneView {
    pub code: String,
    pub label_text: String,
    pub summary: String,
    pub name: String,
    pub unit: String,
    pub quota_units: String,
    pub ratio_text: String,
    pub cost: String,
    pub rotation_days: String,
    pub threshold: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_tiered_forecast_milestone_view(item: &TieredForecastMilestone) -> TieredForecastMilestoneView {
    TieredForecastMilestoneView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        summary: item.summary.clone(),
        name: item.name.clone(),
        unit: text::truncate(&item.unit, 64),
        quota_units: item.quota_units.to_string(),
        ratio_text: format!("{:.2}", item.ratio),
        cost: format!("{:.2}", item.cost),
        rotation_days: item.rotation_days.to_string(),
        threshold: format!("{:.2}", item.threshold),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_forecast_milestone_views(items: &[TieredForecastMilestone]) -> Vec<TieredForecastMilestoneView> {
    let mut views: Vec<TieredForecastMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_forecast_milestone_view(item));
    }
    views
}

pub fn sum_tiered_forecast_milestone_view(view: &TieredForecastMilestoneView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_ratio + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_forecast_milestone("code-616", "label-146");
        let view = to_tiered_forecast_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_forecast_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
