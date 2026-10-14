use super::*;

use acme_kit::text;

/// How screens and exports show a forecast variance.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastVarianceView {
    pub code: String,
    pub title: String,
    pub floor_value_text: String,
    pub approved: String,
    pub fingerprint: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_forecast_variance_view(item: &ForecastVariance) -> ForecastVarianceView {
    ForecastVarianceView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        approved: format!("{}", item.approved),
        fingerprint: item.fingerprint.clone(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_variance_views(items: &[ForecastVariance]) -> Vec<ForecastVarianceView> {
    let mut views: Vec<ForecastVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_variance_view(item));
    }
    views
}

pub fn sum_forecast_variance_view(view: &ForecastVarianceView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_floor_value + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_variance("code-883", "title-241");
        let view = to_forecast_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
