use super::*;

use acme_kit::text;

/// How screens and exports show a forecast policy.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastPolicyView {
    pub code_text: String,
    pub name: String,
    pub maximum: String,
    pub minimum_text: String,
    pub owner_id_text: String,
    pub budget_text: String,
    pub window_days: String,
    pub visible: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_forecast_policy_view(item: &ForecastPolicy) -> ForecastPolicyView {
    ForecastPolicyView {
        code_text: text::truncate(&item.code, 64),
        name: item.name.clone(),
        maximum: format!("{:.2}", item.maximum),
        minimum_text: format!("{:.2}", item.minimum),
        owner_id_text: item.owner_id.clone(),
        budget_text: format!("{:.2}", item.budget),
        window_days: item.window_days.to_string(),
        visible: format!("{}", item.visible),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_policy_views(items: &[ForecastPolicy]) -> Vec<ForecastPolicyView> {
    let mut views: Vec<ForecastPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_policy_view(item));
    }
    views
}

pub fn sum_forecast_policy_view(view: &ForecastPolicyView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_maximum + parsed_minimum + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_policy("code-714", "name-886");
        let view = to_forecast_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
