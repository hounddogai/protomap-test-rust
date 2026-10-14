use super::*;

use acme_kit::text;

/// How screens and exports show a fallback quote threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteThresholdView {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub issuer: String,
    pub floor_value: String,
    pub margin: String,
    pub version_text: String,
    pub status_label: String,
}

pub fn to_fallback_quote_threshold_view(item: &FallbackQuoteThreshold) -> FallbackQuoteThresholdView {
    FallbackQuoteThresholdView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        phone: item.phone.clone(),
        issuer: item.issuer.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        margin: format!("{:.2}", item.margin),
        version_text: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_quote_threshold_views(items: &[FallbackQuoteThreshold]) -> Vec<FallbackQuoteThresholdView> {
    let mut views: Vec<FallbackQuoteThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_quote_threshold_view(item));
    }
    views
}

pub fn sum_fallback_quote_threshold_view(view: &FallbackQuoteThresholdView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_floor_value + parsed_margin + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_quote_threshold("code-915", "name-286");
        let view = to_fallback_quote_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_quote_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
