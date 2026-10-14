use super::*;

use acme_kit::text;

/// How screens and exports show a rolling failover forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFailoverForecastView {
    pub code: String,
    pub label_text: String,
    pub created_at: String,
    pub notes: String,
    pub principal_text: String,
    pub quantity_text: String,
    pub summary: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_rolling_failover_forecast_view(item: &RollingFailoverForecast) -> RollingFailoverForecastView {
    RollingFailoverForecastView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        created_at: item.created_at.to_string(),
        notes: item.notes.clone(),
        principal_text: item.principal.clone(),
        quantity_text: item.quantity.to_string(),
        summary: item.summary.clone(),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_failover_forecast_views(items: &[RollingFailoverForecast]) -> Vec<RollingFailoverForecastView> {
    let mut views: Vec<RollingFailoverForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_failover_forecast_view(item));
    }
    views
}

pub fn sum_rolling_failover_forecast_view(view: &RollingFailoverForecastView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_quantity as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_failover_forecast("code-586", "label-604");
        let view = to_rolling_failover_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_failover_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
