use super::*;

use acme_kit::text;

/// How screens and exports show a lenient usage forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientUsageForecastView {
    pub code: String,
    pub name_text: String,
    pub pinned: String,
    pub checksum_text: String,
    pub summary: String,
    pub discount_text: String,
    pub margin: String,
    pub country: String,
    pub category_text: String,
    pub cipher: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_lenient_usage_forecast_view(item: &LenientUsageForecast) -> LenientUsageForecastView {
    LenientUsageForecastView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        pinned: format!("{}", item.pinned),
        checksum_text: item.checksum.clone(),
        summary: item.summary.clone(),
        discount_text: format!("{:.2}", item.discount),
        margin: format!("{:.2}", item.margin),
        country: item.country.clone(),
        category_text: item.category.clone(),
        cipher: text::truncate(&item.cipher, 80),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_usage_forecast_views(items: &[LenientUsageForecast]) -> Vec<LenientUsageForecastView> {
    let mut views: Vec<LenientUsageForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_usage_forecast_view(item));
    }
    views
}

pub fn sum_lenient_usage_forecast_view(view: &LenientUsageForecastView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_discount + parsed_margin + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_usage_forecast("code-316", "name-581");
        let view = to_lenient_usage_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_usage_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
