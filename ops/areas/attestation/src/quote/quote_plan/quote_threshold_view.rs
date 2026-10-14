use super::*;

use acme_kit::text;

/// How screens and exports show a quote threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteThresholdView {
    pub code_text: String,
    pub title: String,
    pub rate: String,
    pub starts_at_text: String,
    pub locale_text: String,
    pub segment: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_quote_threshold_view(item: &QuoteThreshold) -> QuoteThresholdView {
    QuoteThresholdView {
        code_text: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 80),
        rate: format!("{:.2}", item.rate),
        starts_at_text: item.starts_at.to_string(),
        locale_text: item.locale.clone(),
        segment: item.segment.clone(),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_threshold_views(items: &[QuoteThreshold]) -> Vec<QuoteThresholdView> {
    let mut views: Vec<QuoteThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_threshold_view(item));
    }
    views
}

pub fn sum_quote_threshold_view(view: &QuoteThresholdView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_rate + (parsed_starts_at as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_threshold("code-860", "title-302");
        let view = to_quote_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
