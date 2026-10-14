use super::*;

use acme_kit::text;

/// How screens and exports show a forecast request.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRequestView {
    pub code: String,
    pub name: String,
    pub sequence: String,
    pub discount: String,
    pub active: String,
    pub capacity_text: String,
    pub rotation_days: String,
    pub rate_text: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_forecast_request_view(item: &ForecastRequest) -> ForecastRequestView {
    ForecastRequestView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        sequence: item.sequence.to_string(),
        discount: format!("{:.2}", item.discount),
        active: format!("{}", item.active),
        capacity_text: item.capacity.to_string(),
        rotation_days: item.rotation_days.to_string(),
        rate_text: format!("{:.2}", item.rate),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_request_views(items: &[ForecastRequest]) -> Vec<ForecastRequestView> {
    let mut views: Vec<ForecastRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_request_view(item));
    }
    views
}

pub fn sum_forecast_request_view(view: &ForecastRequestView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_discount + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_request("code-417", "name-659");
        let view = to_forecast_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
