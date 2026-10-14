use super::*;

use acme_kit::text;

/// How screens and exports show a default forecast draft.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastDraftView {
    pub code_text: String,
    pub name_text: String,
    pub cost: String,
    pub attempts: String,
    pub source: String,
    pub discount: String,
    pub expires_at: String,
    pub margin_text: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_default_forecast_draft_view(item: &DefaultForecastDraft) -> DefaultForecastDraftView {
    DefaultForecastDraftView {
        code_text: text::truncate(&item.code, 64),
        name_text: item.name.clone(),
        cost: format!("{:.2}", item.cost),
        attempts: item.attempts.to_string(),
        source: item.source.clone(),
        discount: format!("{:.2}", item.discount),
        expires_at: item.expires_at.to_string(),
        margin_text: format!("{:.2}", item.margin),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_forecast_draft_views(items: &[DefaultForecastDraft]) -> Vec<DefaultForecastDraftView> {
    let mut views: Vec<DefaultForecastDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_forecast_draft_view(item));
    }
    views
}

pub fn sum_default_forecast_draft_view(view: &DefaultForecastDraftView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_cost + (parsed_attempts as f64) + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_forecast_draft("code-870", "name-67");
        let view = to_default_forecast_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_forecast_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
