use super::*;

/// How screens and exports show a region pin forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinForecastView {
    pub code: String,
    pub name: String,
    pub version: String,
    pub ratio: String,
    pub namespace: String,
    pub burst_text: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_region_pin_forecast_view(item: &RegionPinForecast) -> RegionPinForecastView {
    RegionPinForecastView {
        code: item.code.clone(),
        name: item.name.clone(),
        version: item.version.to_string(),
        ratio: format!("{:.2}", item.ratio),
        namespace: item.namespace.clone(),
        burst_text: item.burst.to_string(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_forecast_views(items: &[RegionPinForecast]) -> Vec<RegionPinForecastView> {
    let mut views: Vec<RegionPinForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_forecast_view(item));
    }
    views
}

pub fn sum_region_pin_forecast_view(view: &RegionPinForecastView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_version as f64 + parsed_ratio + (parsed_burst as f64))
}
