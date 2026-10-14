use super::*;

use acme_kit::text;

/// How screens and exports show a token profile.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenProfileView {
    pub code: String,
    pub title_text: String,
    pub source: String,
    pub external_ref: String,
    pub grace_days: String,
    pub timezone: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_token_profile_view(item: &TokenProfile) -> TokenProfileView {
    TokenProfileView {
        code: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        source: item.source.clone(),
        external_ref: item.external_ref.clone(),
        grace_days: item.grace_days.to_string(),
        timezone: item.timezone.clone(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_profile_views(items: &[TokenProfile]) -> Vec<TokenProfileView> {
    let mut views: Vec<TokenProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_profile_view(item));
    }
    views
}

pub fn sum_token_profile_view(view: &TokenProfileView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_profile("code-991", "title-151");
        let view = to_token_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
