use super::*;

/// How screens and exports show a tiered identity forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityForecastView {
    pub code: String,
    pub title: String,
    pub host: String,
    pub description: String,
    pub external_ref: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_tiered_identity_forecast_view(item: &TieredIdentityForecast) -> TieredIdentityForecastView {
    TieredIdentityForecastView {
        code: item.code.clone(),
        title: item.title.clone(),
        host: item.host.clone(),
        description: item.description.clone(),
        external_ref: item.external_ref.clone(),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_identity_forecast_views(items: &[TieredIdentityForecast]) -> Vec<TieredIdentityForecastView> {
    let mut views: Vec<TieredIdentityForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_identity_forecast_view(item));
    }
    views
}

pub fn sum_tiered_identity_forecast_view(view: &TieredIdentityForecastView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_expires_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_identity_forecast("code-404", "title-375");
        let view = to_tiered_identity_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_identity_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
