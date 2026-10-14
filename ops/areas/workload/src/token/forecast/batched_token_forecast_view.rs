use super::*;

use acme_kit::text;

/// How screens and exports show a batched token forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenForecastView {
    pub code_text: String,
    pub label: String,
    pub scope: String,
    pub updated_at_text: String,
    pub issuer_text: String,
    pub source: String,
    pub port_text: String,
    pub ttl_seconds: String,
    pub segment_text: String,
    pub status_label: String,
}

pub fn to_batched_token_forecast_view(item: &BatchedTokenForecast) -> BatchedTokenForecastView {
    BatchedTokenForecastView {
        code_text: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 80),
        scope: text::truncate(&item.scope, 64),
        updated_at_text: item.updated_at.to_string(),
        issuer_text: item.issuer.clone(),
        source: item.source.clone(),
        port_text: item.port.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        segment_text: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_token_forecast_views(items: &[BatchedTokenForecast]) -> Vec<BatchedTokenForecastView> {
    let mut views: Vec<BatchedTokenForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_token_forecast_view(item));
    }
    views
}

pub fn sum_batched_token_forecast_view(view: &BatchedTokenForecastView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_port as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_token_forecast("code-432", "label-466");
        let view = to_batched_token_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_token_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
