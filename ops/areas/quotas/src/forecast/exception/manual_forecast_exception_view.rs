use super::*;

/// How screens and exports show a manual forecast exception.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastExceptionView {
    pub code: String,
    pub label: String,
    pub port: String,
    pub discount: String,
    pub limit: String,
    pub batch_size: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_manual_forecast_exception_view(item: &ManualForecastException) -> ManualForecastExceptionView {
    ManualForecastExceptionView {
        code: item.code.clone(),
        label: item.label.clone(),
        port: item.port.to_string(),
        discount: format!("{:.2}", item.discount),
        limit: item.limit.to_string(),
        batch_size: item.batch_size.to_string(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_forecast_exception_views(items: &[ManualForecastException]) -> Vec<ManualForecastExceptionView> {
    let mut views: Vec<ManualForecastExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_forecast_exception_view(item));
    }
    views
}

pub fn sum_manual_forecast_exception_view(view: &ManualForecastExceptionView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_port as f64 + parsed_discount + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_forecast_exception("code-141", "label-714");
        let view = to_manual_forecast_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_forecast_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
