use super::*;

/// How screens and exports show a token manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenManifestView {
    pub code_text: String,
    pub name: String,
    pub score_text: String,
    pub source_text: String,
    pub priority_text: String,
    pub status_label: String,
}

pub fn to_token_manifest_view(item: &TokenManifest) -> TokenManifestView {
    TokenManifestView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        score_text: format!("{:.2}", item.score),
        source_text: item.source.clone(),
        priority_text: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_manifest_views(items: &[TokenManifest]) -> Vec<TokenManifestView> {
    let mut views: Vec<TokenManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_manifest_view(item));
    }
    views
}

pub fn sum_token_manifest_view(view: &TokenManifestView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_score + (parsed_priority as f64))
}
