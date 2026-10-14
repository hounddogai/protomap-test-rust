use super::*;

/// How screens and exports show a usage forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageForecastView {
    pub code: String,
    pub name: String,
    pub rate: String,
    pub revision_text: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_usage_forecast_view(item: &UsageForecast) -> UsageForecastView {
    UsageForecastView {
        code: item.code.clone(),
        name: item.name.clone(),
        rate: format!("{:.2}", item.rate),
        revision_text: item.revision.to_string(),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_forecast_views(items: &[UsageForecast]) -> Vec<UsageForecastView> {
    let mut views: Vec<UsageForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_forecast_view(item));
    }
    views
}

pub fn sum_usage_forecast_view(view: &UsageForecastView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_rate + (parsed_revision as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_forecast("code-700", "name-19");
        let view = to_usage_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
