use super::*;

use acme_kit::text;

/// How screens and exports show a failover forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverForecastView {
    pub code_text: String,
    pub name: String,
    pub grace_days_text: String,
    pub unit: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_failover_forecast_view(item: &FailoverForecast) -> FailoverForecastView {
    FailoverForecastView {
        code_text: text::truncate(&item.code, 64),
        name: item.name.clone(),
        grace_days_text: item.grace_days.to_string(),
        unit: item.unit.clone(),
        description: text::truncate(&item.description, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_forecast_views(items: &[FailoverForecast]) -> Vec<FailoverForecastView> {
    let mut views: Vec<FailoverForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_forecast_view(item));
    }
    views
}

pub fn sum_failover_forecast_view(view: &FailoverForecastView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_forecast("code-104", "name-796");
        let view = to_failover_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
