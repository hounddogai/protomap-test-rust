use super::*;

use acme_kit::text;

/// How screens and exports show a fallback budget burn forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnForecastView {
    pub code: String,
    pub name: String,
    pub audience_text: String,
    pub minimum: String,
    pub reviewer: String,
    pub title_text: String,
    pub category: String,
    pub batch_size_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_fallback_budget_burn_forecast_view(item: &FallbackBudgetBurnForecast) -> FallbackBudgetBurnForecastView {
    FallbackBudgetBurnForecastView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        audience_text: item.audience.clone(),
        minimum: format!("{:.2}", item.minimum),
        reviewer: item.reviewer.clone(),
        title_text: text::truncate(&item.title, 64),
        category: item.category.clone(),
        batch_size_text: item.batch_size.to_string(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_budget_burn_forecast_views(
    items: &[FallbackBudgetBurnForecast],
) -> Vec<FallbackBudgetBurnForecastView> {
    let mut views: Vec<FallbackBudgetBurnForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_budget_burn_forecast_view(item));
    }
    views
}

pub fn sum_fallback_budget_burn_forecast_view(view: &FallbackBudgetBurnForecastView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_minimum + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_budget_burn_forecast("code-134", "name-367");
        let view = to_fallback_budget_burn_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_budget_burn_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
