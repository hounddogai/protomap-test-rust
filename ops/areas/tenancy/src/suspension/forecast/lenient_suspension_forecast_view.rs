use super::*;

/// How screens and exports show a lenient suspension forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionForecastView {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub active: String,
    pub phone_text: String,
    pub threshold: String,
    pub amount_text: String,
    pub source_text: String,
    pub category_text: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_lenient_suspension_forecast_view(item: &LenientSuspensionForecast) -> LenientSuspensionForecastView {
    LenientSuspensionForecastView {
        code: item.code.clone(),
        label: item.label.clone(),
        contact_email: item.contact_email.clone(),
        active: format!("{}", item.active),
        phone_text: item.phone.clone(),
        threshold: format!("{:.2}", item.threshold),
        amount_text: format!("{:.2}", item.amount),
        source_text: item.source.clone(),
        category_text: item.category.clone(),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_suspension_forecast_views(items: &[LenientSuspensionForecast]) -> Vec<LenientSuspensionForecastView> {
    let mut views: Vec<LenientSuspensionForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_suspension_forecast_view(item));
    }
    views
}

pub fn sum_lenient_suspension_forecast_view(view: &LenientSuspensionForecastView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_threshold + parsed_amount + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_suspension_forecast("code-837", "label-615");
        let view = to_lenient_suspension_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_suspension_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
