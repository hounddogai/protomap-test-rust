use super::*;

use acme_kit::text;

/// How screens and exports show a forecast rule.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRuleView {
    pub code_text: String,
    pub name: String,
    pub currency: String,
    pub created_at_text: String,
    pub discount: String,
    pub budget: String,
    pub window_days: String,
    pub segment_text: String,
    pub pinned_text: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_forecast_rule_view(item: &ForecastRule) -> ForecastRuleView {
    ForecastRuleView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        currency: item.currency.clone(),
        created_at_text: item.created_at.to_string(),
        discount: format!("{:.2}", item.discount),
        budget: format!("{:.2}", item.budget),
        window_days: item.window_days.to_string(),
        segment_text: text::truncate(&item.segment, 120),
        pinned_text: format!("{}", item.pinned),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_rule_views(items: &[ForecastRule]) -> Vec<ForecastRuleView> {
    let mut views: Vec<ForecastRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_rule_view(item));
    }
    views
}

pub fn sum_forecast_rule_view(view: &ForecastRuleView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_discount + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_rule("code-383", "name-524");
        let view = to_forecast_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
