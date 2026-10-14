use super::*;

/// How screens and exports show a token bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenBundleView {
    pub code: String,
    pub label: String,
    pub expires_at_text: String,
    pub external_ref: String,
    pub retries: String,
    pub title: String,
    pub phone: String,
    pub floor_value_text: String,
    pub source: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_token_bundle_view(item: &TokenBundle) -> TokenBundleView {
    TokenBundleView {
        code: item.code.clone(),
        label: item.label.clone(),
        expires_at_text: item.expires_at.to_string(),
        external_ref: item.external_ref.clone(),
        retries: item.retries.to_string(),
        title: item.title.clone(),
        phone: item.phone.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        source: item.source.clone(),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_bundle_views(items: &[TokenBundle]) -> Vec<TokenBundleView> {
    let mut views: Vec<TokenBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_bundle_view(item));
    }
    views
}

pub fn sum_token_bundle_view(view: &TokenBundleView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_retries as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_bundle("code-899", "label-167");
        let view = to_token_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
