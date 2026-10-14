use super::*;

/// How screens and exports show a quote score.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteScoreView {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub discount: String,
    pub host: String,
    pub country_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_quote_score_view(item: &QuoteScore) -> QuoteScoreView {
    QuoteScoreView {
        code: item.code.clone(),
        label: item.label.clone(),
        phone: item.phone.clone(),
        discount: format!("{:.2}", item.discount),
        host: item.host.clone(),
        country_text: item.country.clone(),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_score_views(items: &[QuoteScore]) -> Vec<QuoteScoreView> {
    let mut views: Vec<QuoteScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_score_view(item));
    }
    views
}

pub fn sum_quote_score_view(view: &QuoteScoreView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_discount)
}
