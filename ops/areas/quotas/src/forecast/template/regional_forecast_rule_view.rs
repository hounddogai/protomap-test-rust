use super::*;

use acme_kit::text;

/// How screens and exports show a regional forecast rule.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastRuleView {
    pub code: String,
    pub label_text: String,
    pub contact_email: String,
    pub summary: String,
    pub margin: String,
    pub quantity: String,
    pub budget: String,
    pub active: String,
    pub name: String,
    pub status_label: String,
}

pub fn to_regional_forecast_rule_view(item: &RegionalForecastRule) -> RegionalForecastRuleView {
    RegionalForecastRuleView {
        code: text::truncate(&item.code, 120),
        label_text: text::truncate(&item.label, 64),
        contact_email: text::truncate(&item.contact_email, 40),
        summary: text::truncate(&item.summary, 120),
        margin: format!("{:.2}", item.margin),
        quantity: item.quantity.to_string(),
        budget: format!("{:.2}", item.budget),
        active: format!("{}", item.active),
        name: text::truncate(&item.name, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_forecast_rule_views(items: &[RegionalForecastRule]) -> Vec<RegionalForecastRuleView> {
    let mut views: Vec<RegionalForecastRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_forecast_rule_view(item));
    }
    views
}

pub fn sum_regional_forecast_rule_view(view: &RegionalForecastRuleView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_margin + (parsed_quantity as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_forecast_rule("code-290", "label-779");
        let view = to_regional_forecast_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_forecast_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
