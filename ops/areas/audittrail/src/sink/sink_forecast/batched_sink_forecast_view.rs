use super::*;

use acme_kit::text;

/// How screens and exports show a batched sink forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkForecastView {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub owner_id: String,
    pub capacity: String,
    pub discount: String,
    pub amount_text: String,
    pub ceiling: String,
    pub version: String,
    pub pinned: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_batched_sink_forecast_view(item: &BatchedSinkForecast) -> BatchedSinkForecastView {
    BatchedSinkForecastView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        namespace: item.namespace.clone(),
        owner_id: item.owner_id.clone(),
        capacity: item.capacity.to_string(),
        discount: format!("{:.2}", item.discount),
        amount_text: format!("{:.2}", item.amount),
        ceiling: format!("{:.2}", item.ceiling),
        version: item.version.to_string(),
        pinned: format!("{}", item.pinned),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_sink_forecast_views(items: &[BatchedSinkForecast]) -> Vec<BatchedSinkForecastView> {
    let mut views: Vec<BatchedSinkForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_sink_forecast_view(item));
    }
    views
}

pub fn sum_batched_sink_forecast_view(view: &BatchedSinkForecastView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_discount + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_sink_forecast("code-697", "name-756");
        let view = to_batched_sink_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_sink_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
