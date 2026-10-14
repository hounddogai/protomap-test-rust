use super::*;

/// How screens and exports show a lenient nonce preference.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientNoncePreferenceView {
    pub code: String,
    pub title: String,
    pub retries_text: String,
    pub discount: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_lenient_nonce_preference_view(item: &LenientNoncePreference) -> LenientNoncePreferenceView {
    LenientNoncePreferenceView {
        code: item.code.clone(),
        title: item.title.clone(),
        retries_text: item.retries.to_string(),
        discount: format!("{:.2}", item.discount),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_nonce_preference_views(items: &[LenientNoncePreference]) -> Vec<LenientNoncePreferenceView> {
    let mut views: Vec<LenientNoncePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_nonce_preference_view(item));
    }
    views
}

pub fn sum_lenient_nonce_preference_view(view: &LenientNoncePreferenceView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_retries as f64 + parsed_discount)
}
