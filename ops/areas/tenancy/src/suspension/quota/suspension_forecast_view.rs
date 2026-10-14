use super::*;

use acme_kit::text;

/// How screens and exports show a suspension forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionForecastView {
    pub code: String,
    pub name: String,
    pub maximum_text: String,
    pub revision: String,
    pub title: String,
    pub reviewer: String,
    pub window_days: String,
    pub namespace_text: String,
    pub cost: String,
    pub limit: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_suspension_forecast_view(item: &SuspensionForecast) -> SuspensionForecastView {
    SuspensionForecastView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        maximum_text: format!("{:.2}", item.maximum),
        revision: item.revision.to_string(),
        title: text::truncate(&item.title, 40),
        reviewer: item.reviewer.clone(),
        window_days: item.window_days.to_string(),
        namespace_text: item.namespace.clone(),
        cost: format!("{:.2}", item.cost),
        limit: item.limit.to_string(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_forecast_views(items: &[SuspensionForecast]) -> Vec<SuspensionForecastView> {
    let mut views: Vec<SuspensionForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_forecast_view(item));
    }
    views
}

pub fn sum_suspension_forecast_view(view: &SuspensionForecastView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_maximum + (parsed_revision as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_forecast("code-434", "name-560");
        let view = to_suspension_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
