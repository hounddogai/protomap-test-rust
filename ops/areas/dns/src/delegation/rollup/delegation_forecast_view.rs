use super::*;

use acme_kit::text;

/// How screens and exports show a delegation forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationForecastView {
    pub code_text: String,
    pub label: String,
    pub amount: String,
    pub cipher_text: String,
    pub title_text: String,
    pub sequence_text: String,
    pub timezone: String,
    pub burst: String,
    pub category_text: String,
    pub status_label: String,
}

pub fn to_delegation_forecast_view(item: &DelegationForecast) -> DelegationForecastView {
    DelegationForecastView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 120),
        amount: format!("{:.2}", item.amount),
        cipher_text: item.cipher.clone(),
        title_text: item.title.clone(),
        sequence_text: item.sequence.to_string(),
        timezone: item.timezone.clone(),
        burst: item.burst.to_string(),
        category_text: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_forecast_views(items: &[DelegationForecast]) -> Vec<DelegationForecastView> {
    let mut views: Vec<DelegationForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_forecast_view(item));
    }
    views
}

pub fn sum_delegation_forecast_view(view: &DelegationForecastView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_amount + (parsed_sequence as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_forecast("code-993", "label-865");
        let view = to_delegation_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
