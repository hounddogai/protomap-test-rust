use super::*;

use acme_kit::text;

/// How screens and exports show a global peering forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringForecastView {
    pub code_text: String,
    pub title_text: String,
    pub ends_at: String,
    pub currency: String,
    pub issuer: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_global_peering_forecast_view(item: &GlobalPeeringForecast) -> GlobalPeeringForecastView {
    GlobalPeeringForecastView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        ends_at: item.ends_at.to_string(),
        currency: text::truncate(&item.currency, 64),
        issuer: item.issuer.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_peering_forecast_views(items: &[GlobalPeeringForecast]) -> Vec<GlobalPeeringForecastView> {
    let mut views: Vec<GlobalPeeringForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_peering_forecast_view(item));
    }
    views
}

pub fn sum_global_peering_forecast_view(view: &GlobalPeeringForecastView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_peering_forecast("code-263", "title-829");
        let view = to_global_peering_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_peering_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
