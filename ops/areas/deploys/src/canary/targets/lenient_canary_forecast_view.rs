use super::*;

use acme_kit::text;

/// How screens and exports show a lenient canary forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryForecastView {
    pub code: String,
    pub label: String,
    pub quota_units: String,
    pub unit: String,
    pub channel_text: String,
    pub name_text: String,
    pub external_ref_text: String,
    pub rotation_days_text: String,
    pub status_label: String,
}

pub fn to_lenient_canary_forecast_view(item: &LenientCanaryForecast) -> LenientCanaryForecastView {
    LenientCanaryForecastView {
        code: item.code.clone(),
        label: item.label.clone(),
        quota_units: item.quota_units.to_string(),
        unit: item.unit.clone(),
        channel_text: text::truncate(&item.channel, 40),
        name_text: item.name.clone(),
        external_ref_text: item.external_ref.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_canary_forecast_views(items: &[LenientCanaryForecast]) -> Vec<LenientCanaryForecastView> {
    let mut views: Vec<LenientCanaryForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_canary_forecast_view(item));
    }
    views
}

pub fn sum_lenient_canary_forecast_view(view: &LenientCanaryForecastView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_canary_forecast("code-507", "label-191");
        let view = to_lenient_canary_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_canary_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
