use super::*;

use acme_kit::text;

/// How screens and exports show a isolation forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationForecastView {
    pub code: String,
    pub title_text: String,
    pub ends_at: String,
    pub batch_size: String,
    pub quota_units: String,
    pub archived: String,
    pub ttl_seconds: String,
    pub discount: String,
    pub port: String,
    pub name: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_isolation_forecast_view(item: &IsolationForecast) -> IsolationForecastView {
    IsolationForecastView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        ends_at: item.ends_at.to_string(),
        batch_size: item.batch_size.to_string(),
        quota_units: item.quota_units.to_string(),
        archived: format!("{}", item.archived),
        ttl_seconds: item.ttl_seconds.to_string(),
        discount: format!("{:.2}", item.discount),
        port: item.port.to_string(),
        name: item.name.clone(),
        source: text::truncate(&item.source, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_forecast_views(items: &[IsolationForecast]) -> Vec<IsolationForecastView> {
    let mut views: Vec<IsolationForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_forecast_view(item));
    }
    views
}

pub fn sum_isolation_forecast_view(view: &IsolationForecastView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_batch_size as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_forecast("code-473", "title-647");
        let view = to_isolation_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
