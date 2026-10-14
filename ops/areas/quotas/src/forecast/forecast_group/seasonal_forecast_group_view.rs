use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal forecast group.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastGroupView {
    pub code: String,
    pub label: String,
    pub currency_text: String,
    pub duration_minutes_text: String,
    pub namespace_text: String,
    pub segment: String,
    pub checksum: String,
    pub ttl_seconds: String,
    pub quantity: String,
    pub host_text: String,
    pub status_label: String,
}

pub fn to_seasonal_forecast_group_view(item: &SeasonalForecastGroup) -> SeasonalForecastGroupView {
    SeasonalForecastGroupView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        currency_text: item.currency.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        namespace_text: item.namespace.clone(),
        segment: text::truncate(&item.segment, 64),
        checksum: text::truncate(&item.checksum, 80),
        ttl_seconds: item.ttl_seconds.to_string(),
        quantity: item.quantity.to_string(),
        host_text: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_forecast_group_views(items: &[SeasonalForecastGroup]) -> Vec<SeasonalForecastGroupView> {
    let mut views: Vec<SeasonalForecastGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_forecast_group_view(item));
    }
    views
}

pub fn sum_seasonal_forecast_group_view(view: &SeasonalForecastGroupView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_ttl_seconds as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_forecast_group("code-93", "label-421");
        let view = to_seasonal_forecast_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_forecast_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
