use super::*;

use acme_kit::text;

/// How screens and exports show a token digest.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenDigestView {
    pub code: String,
    pub name_text: String,
    pub audience: String,
    pub owner_id: String,
    pub reviewer: String,
    pub minimum_text: String,
    pub port: String,
    pub weight: String,
    pub margin: String,
    pub discount_text: String,
    pub status_label: String,
}

pub fn to_token_digest_view(item: &TokenDigest) -> TokenDigestView {
    TokenDigestView {
        code: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        audience: item.audience.clone(),
        owner_id: item.owner_id.clone(),
        reviewer: item.reviewer.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        port: item.port.to_string(),
        weight: format!("{:.2}", item.weight),
        margin: format!("{:.2}", item.margin),
        discount_text: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_digest_views(items: &[TokenDigest]) -> Vec<TokenDigestView> {
    let mut views: Vec<TokenDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_digest_view(item));
    }
    views
}

pub fn sum_token_digest_view(view: &TokenDigestView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_minimum + (parsed_port as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_digest("code-250", "name-639");
        let view = to_token_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
