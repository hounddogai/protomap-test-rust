use super::*;

use acme_kit::text;

/// How screens and exports show a lenient redaction forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionForecastView {
    pub code: String,
    pub label: String,
    pub unit_text: String,
    pub batch_size: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_lenient_redaction_forecast_view(item: &LenientRedactionForecast) -> LenientRedactionForecastView {
    LenientRedactionForecastView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        unit_text: item.unit.clone(),
        batch_size: item.batch_size.to_string(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_redaction_forecast_views(items: &[LenientRedactionForecast]) -> Vec<LenientRedactionForecastView> {
    let mut views: Vec<LenientRedactionForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_redaction_forecast_view(item));
    }
    views
}

pub fn sum_lenient_redaction_forecast_view(view: &LenientRedactionForecastView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_redaction_forecast("code-12", "label-678");
        let view = to_lenient_redaction_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_redaction_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
