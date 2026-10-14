use super::*;

use acme_kit::text;

/// How screens and exports show a quote baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBaselineView {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub margin_text: String,
    pub score: String,
    pub window_days: String,
    pub rotation_days: String,
    pub maximum_text: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_quote_baseline_view(item: &QuoteBaseline) -> QuoteBaselineView {
    QuoteBaselineView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        notes: item.notes.clone(),
        margin_text: format!("{:.2}", item.margin),
        score: format!("{:.2}", item.score),
        window_days: item.window_days.to_string(),
        rotation_days: item.rotation_days.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        channel: text::truncate(&item.channel, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_baseline_views(items: &[QuoteBaseline]) -> Vec<QuoteBaselineView> {
    let mut views: Vec<QuoteBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_baseline_view(item));
    }
    views
}

pub fn sum_quote_baseline_view(view: &QuoteBaselineView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_margin + parsed_score + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_baseline("code-559", "title-843");
        let view = to_quote_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
