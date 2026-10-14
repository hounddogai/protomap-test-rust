use super::*;

use acme_kit::text;

/// How screens and exports show a flow log forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogForecastView {
    pub code_text: String,
    pub title: String,
    pub priority_text: String,
    pub currency: String,
    pub category_text: String,
    pub active: String,
    pub discount: String,
    pub port_text: String,
    pub retries_text: String,
    pub visible: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_flow_log_forecast_view(item: &FlowLogForecast) -> FlowLogForecastView {
    FlowLogForecastView {
        code_text: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 80),
        priority_text: item.priority.to_string(),
        currency: text::truncate(&item.currency, 64),
        category_text: item.category.clone(),
        active: format!("{}", item.active),
        discount: format!("{:.2}", item.discount),
        port_text: item.port.to_string(),
        retries_text: item.retries.to_string(),
        visible: format!("{}", item.visible),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_forecast_views(items: &[FlowLogForecast]) -> Vec<FlowLogForecastView> {
    let mut views: Vec<FlowLogForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_forecast_view(item));
    }
    views
}

pub fn sum_flow_log_forecast_view(view: &FlowLogForecastView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_priority as f64 + parsed_discount + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_forecast("code-842", "title-396");
        let view = to_flow_log_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
