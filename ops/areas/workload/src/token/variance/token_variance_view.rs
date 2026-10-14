use super::*;

use acme_kit::text;

/// How screens and exports show a token variance.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenVarianceView {
    pub code_text: String,
    pub title: String,
    pub principal: String,
    pub source_text: String,
    pub external_ref: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_token_variance_view(item: &TokenVariance) -> TokenVarianceView {
    TokenVarianceView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        principal: text::truncate(&item.principal, 64),
        source_text: text::truncate(&item.source, 64),
        external_ref: item.external_ref.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_variance_views(items: &[TokenVariance]) -> Vec<TokenVarianceView> {
    let mut views: Vec<TokenVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_variance_view(item));
    }
    views
}

pub fn sum_token_variance_view(view: &TokenVarianceView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}
