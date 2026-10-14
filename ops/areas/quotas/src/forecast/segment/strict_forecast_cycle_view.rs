use super::*;

use acme_kit::text;

/// How screens and exports show a strict forecast cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastCycleView {
    pub code: String,
    pub label: String,
    pub quantity_text: String,
    pub currency: String,
    pub notes_text: String,
    pub status_label: String,
}

pub fn to_strict_forecast_cycle_view(item: &StrictForecastCycle) -> StrictForecastCycleView {
    StrictForecastCycleView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        quantity_text: item.quantity.to_string(),
        currency: item.currency.clone(),
        notes_text: text::truncate(&item.notes, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_forecast_cycle_views(items: &[StrictForecastCycle]) -> Vec<StrictForecastCycleView> {
    let mut views: Vec<StrictForecastCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_forecast_cycle_view(item));
    }
    views
}

pub fn sum_strict_forecast_cycle_view(view: &StrictForecastCycleView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_forecast_cycle("code-661", "label-941");
        let view = to_strict_forecast_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_forecast_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
