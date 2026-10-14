use super::*;

use acme_kit::text;

/// How screens and exports show a token override.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenOverrideView {
    pub code: String,
    pub title_text: String,
    pub starts_at: String,
    pub ratio: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_token_override_view(item: &TokenOverride) -> TokenOverrideView {
    TokenOverrideView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        starts_at: item.starts_at.to_string(),
        ratio: format!("{:.2}", item.ratio),
        phone: text::truncate(&item.phone, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_override_views(items: &[TokenOverride]) -> Vec<TokenOverrideView> {
    let mut views: Vec<TokenOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_override_view(item));
    }
    views
}

pub fn sum_token_override_view(view: &TokenOverrideView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_override("code-257", "title-652");
        let view = to_token_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
