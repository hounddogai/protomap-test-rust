use super::*;

use acme_kit::text;

/// How screens and exports show a zone forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneForecastView {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub namespace: String,
    pub capacity: String,
    pub sequence: String,
    pub minimum_text: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_zone_forecast_view(item: &ZoneForecast) -> ZoneForecastView {
    ZoneForecastView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        reviewer: item.reviewer.clone(),
        namespace: item.namespace.clone(),
        capacity: item.capacity.to_string(),
        sequence: item.sequence.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        source: text::truncate(&item.source, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_forecast_views(items: &[ZoneForecast]) -> Vec<ZoneForecastView> {
    let mut views: Vec<ZoneForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_forecast_view(item));
    }
    views
}

pub fn sum_zone_forecast_view(view: &ZoneForecastView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_sequence as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_forecast("code-131", "name-397");
        let view = to_zone_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
