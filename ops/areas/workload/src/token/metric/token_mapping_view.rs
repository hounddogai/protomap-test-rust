use super::*;

/// How screens and exports show a token mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenMappingView {
    pub code: String,
    pub title: String,
    pub approved: String,
    pub batch_size: String,
    pub margin: String,
    pub scope: String,
    pub sequence_text: String,
    pub grace_days_text: String,
    pub status_label: String,
}

pub fn to_token_mapping_view(item: &TokenMapping) -> TokenMappingView {
    TokenMappingView {
        code: item.code.clone(),
        title: item.title.clone(),
        approved: format!("{}", item.approved),
        batch_size: item.batch_size.to_string(),
        margin: format!("{:.2}", item.margin),
        scope: item.scope.clone(),
        sequence_text: item.sequence.to_string(),
        grace_days_text: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_mapping_views(items: &[TokenMapping]) -> Vec<TokenMappingView> {
    let mut views: Vec<TokenMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_mapping_view(item));
    }
    views
}

pub fn sum_token_mapping_view(view: &TokenMappingView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_margin + (parsed_sequence as f64))
}
