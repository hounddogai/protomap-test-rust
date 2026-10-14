use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainForecastView {
    pub code_text: String,
    pub name: String,
    pub amount: String,
    pub ends_at_text: String,
    pub score_text: String,
    pub capacity: String,
    pub channel: String,
    pub burst: String,
    pub port: String,
    pub duration_minutes: String,
    pub country_text: String,
    pub status_label: String,
}

pub fn to_trust_domain_forecast_view(item: &TrustDomainForecast) -> TrustDomainForecastView {
    TrustDomainForecastView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 120),
        amount: format!("{:.2}", item.amount),
        ends_at_text: item.ends_at.to_string(),
        score_text: format!("{:.2}", item.score),
        capacity: item.capacity.to_string(),
        channel: item.channel.clone(),
        burst: item.burst.to_string(),
        port: item.port.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        country_text: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_forecast_views(items: &[TrustDomainForecast]) -> Vec<TrustDomainForecastView> {
    let mut views: Vec<TrustDomainForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_forecast_view(item));
    }
    views
}

pub fn sum_trust_domain_forecast_view(view: &TrustDomainForecastView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_amount + (parsed_ends_at as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_forecast("code-479", "name-683");
        let view = to_trust_domain_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
