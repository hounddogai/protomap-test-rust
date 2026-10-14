use super::*;

use acme_kit::text;

/// How screens and exports show a redaction forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionForecastView {
    pub code_text: String,
    pub name: String,
    pub starts_at: String,
    pub category: String,
    pub updated_at_text: String,
    pub status_label: String,
}

pub fn to_redaction_forecast_view(item: &RedactionForecast) -> RedactionForecastView {
    RedactionForecastView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        starts_at: item.starts_at.to_string(),
        category: text::truncate(&item.category, 64),
        updated_at_text: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_forecast_views(items: &[RedactionForecast]) -> Vec<RedactionForecastView> {
    let mut views: Vec<RedactionForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_forecast_view(item));
    }
    views
}

pub fn sum_redaction_forecast_view(view: &RedactionForecastView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_redaction_forecast("code-671", "name-645");
        let view = to_redaction_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_redaction_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
