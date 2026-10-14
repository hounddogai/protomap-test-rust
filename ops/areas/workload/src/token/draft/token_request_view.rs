use super::*;

use acme_kit::text;

/// How screens and exports show a token request.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenRequestView {
    pub code: String,
    pub label: String,
    pub weight: String,
    pub created_at: String,
    pub fingerprint_text: String,
    pub contact_email: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_token_request_view(item: &TokenRequest) -> TokenRequestView {
    TokenRequestView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        weight: format!("{:.2}", item.weight),
        created_at: item.created_at.to_string(),
        fingerprint_text: item.fingerprint.clone(),
        contact_email: item.contact_email.clone(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_request_views(items: &[TokenRequest]) -> Vec<TokenRequestView> {
    let mut views: Vec<TokenRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_request_view(item));
    }
    views
}

pub fn sum_token_request_view(view: &TokenRequestView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_weight + (parsed_created_at as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_request("code-439", "label-636");
        let view = to_token_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
