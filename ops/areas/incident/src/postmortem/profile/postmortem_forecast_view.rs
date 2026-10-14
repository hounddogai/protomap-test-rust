use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemForecastView {
    pub code: String,
    pub title_text: String,
    pub margin: String,
    pub label_text: String,
    pub amount: String,
    pub owner_id_text: String,
    pub status_label: String,
}

pub fn to_postmortem_forecast_view(item: &PostmortemForecast) -> PostmortemForecastView {
    PostmortemForecastView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 120),
        margin: format!("{:.2}", item.margin),
        label_text: item.label.clone(),
        amount: format!("{:.2}", item.amount),
        owner_id_text: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_forecast_views(items: &[PostmortemForecast]) -> Vec<PostmortemForecastView> {
    let mut views: Vec<PostmortemForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_forecast_view(item));
    }
    views
}

pub fn sum_postmortem_forecast_view(view: &PostmortemForecastView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_margin + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_postmortem_forecast("code-321", "title-824");
        let view = to_postmortem_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_postmortem_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
