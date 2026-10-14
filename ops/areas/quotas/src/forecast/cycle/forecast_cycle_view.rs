use super::*;

use acme_kit::text;

/// How screens and exports show a forecast cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastCycleView {
    pub code: String,
    pub label: String,
    pub rate: String,
    pub limit: String,
    pub title_text: String,
    pub scope: String,
    pub ratio_text: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_forecast_cycle_view(item: &ForecastCycle) -> ForecastCycleView {
    ForecastCycleView {
        code: item.code.clone(),
        label: item.label.clone(),
        rate: format!("{:.2}", item.rate),
        limit: item.limit.to_string(),
        title_text: item.title.clone(),
        scope: text::truncate(&item.scope, 64),
        ratio_text: format!("{:.2}", item.ratio),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_cycle_views(items: &[ForecastCycle]) -> Vec<ForecastCycleView> {
    let mut views: Vec<ForecastCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_cycle_view(item));
    }
    views
}

pub fn sum_forecast_cycle_view(view: &ForecastCycleView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_rate + (parsed_limit as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_cycle("code-393", "label-984");
        let view = to_forecast_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
