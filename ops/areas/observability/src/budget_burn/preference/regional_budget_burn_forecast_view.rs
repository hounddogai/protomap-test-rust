use super::*;

use acme_kit::text;

/// How screens and exports show a regional budget burn forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnForecastView {
    pub code_text: String,
    pub title_text: String,
    pub ttl_seconds: String,
    pub cipher: String,
    pub contact_email: String,
    pub quantity: String,
    pub active_text: String,
    pub checksum: String,
    pub priority: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_regional_budget_burn_forecast_view(item: &RegionalBudgetBurnForecast) -> RegionalBudgetBurnForecastView {
    RegionalBudgetBurnForecastView {
        code_text: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        cipher: item.cipher.clone(),
        contact_email: item.contact_email.clone(),
        quantity: item.quantity.to_string(),
        active_text: format!("{}", item.active),
        checksum: text::truncate(&item.checksum, 64),
        priority: item.priority.to_string(),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_budget_burn_forecast_views(
    items: &[RegionalBudgetBurnForecast],
) -> Vec<RegionalBudgetBurnForecastView> {
    let mut views: Vec<RegionalBudgetBurnForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_budget_burn_forecast_view(item));
    }
    views
}

pub fn sum_regional_budget_burn_forecast_view(view: &RegionalBudgetBurnForecastView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_quantity as f64) + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_budget_burn_forecast("code-402", "title-806");
        let view = to_regional_budget_burn_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_budget_burn_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
