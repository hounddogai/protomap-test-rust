use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled forecast preference.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastPreferenceView {
    pub code: String,
    pub label_text: String,
    pub floor_value: String,
    pub unit: String,
    pub description_text: String,
    pub budget_text: String,
    pub checksum: String,
    pub timezone_text: String,
    pub principal: String,
    pub key_id: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_scheduled_forecast_preference_view(item: &ScheduledForecastPreference) -> ScheduledForecastPreferenceView {
    ScheduledForecastPreferenceView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        unit: item.unit.clone(),
        description_text: text::truncate(&item.description, 80),
        budget_text: format!("{:.2}", item.budget),
        checksum: text::truncate(&item.checksum, 120),
        timezone_text: item.timezone.clone(),
        principal: item.principal.clone(),
        key_id: item.key_id.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_forecast_preference_views(
    items: &[ScheduledForecastPreference],
) -> Vec<ScheduledForecastPreferenceView> {
    let mut views: Vec<ScheduledForecastPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_forecast_preference_view(item));
    }
    views
}

pub fn sum_scheduled_forecast_preference_view(view: &ScheduledForecastPreferenceView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_floor_value + parsed_budget + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_forecast_preference("code-833", "label-80");
        let view = to_scheduled_forecast_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_forecast_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
