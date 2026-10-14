use super::*;

use acme_kit::text;

/// How screens and exports show a token forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenForecastView {
    pub code: String,
    pub title_text: String,
    pub label_text: String,
    pub floor_value: String,
    pub visible: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_token_forecast_view(item: &TokenForecast) -> TokenForecastView {
    TokenForecastView {
        code: text::truncate(&item.code, 40),
        title_text: text::truncate(&item.title, 64),
        label_text: item.label.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        visible: format!("{}", item.visible),
        channel: text::truncate(&item.channel, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_forecast_views(items: &[TokenForecast]) -> Vec<TokenForecastView> {
    let mut views: Vec<TokenForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_forecast_view(item));
    }
    views
}

pub fn sum_token_forecast_view(view: &TokenForecastView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_forecast("code-325", "title-841");
        let view = to_token_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
