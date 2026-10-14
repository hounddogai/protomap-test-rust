use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal page forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPageForecastView {
    pub code: String,
    pub label_text: String,
    pub priority_text: String,
    pub amount_text: String,
    pub owner_id: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_seasonal_page_forecast_view(item: &SeasonalPageForecast) -> SeasonalPageForecastView {
    SeasonalPageForecastView {
        code: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        priority_text: item.priority.to_string(),
        amount_text: format!("{:.2}", item.amount),
        owner_id: text::truncate(&item.owner_id, 80),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_page_forecast_views(items: &[SeasonalPageForecast]) -> Vec<SeasonalPageForecastView> {
    let mut views: Vec<SeasonalPageForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_page_forecast_view(item));
    }
    views
}

pub fn sum_seasonal_page_forecast_view(view: &SeasonalPageForecastView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_priority as f64 + parsed_amount + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_page_forecast("code-403", "label-617");
        let view = to_seasonal_page_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_page_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
