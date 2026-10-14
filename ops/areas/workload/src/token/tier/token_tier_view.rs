use super::*;

use acme_kit::text;

/// How screens and exports show a token tier.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenTierView {
    pub code: String,
    pub title: String,
    pub margin_text: String,
    pub quantity: String,
    pub port: String,
    pub weight: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_token_tier_view(item: &TokenTier) -> TokenTierView {
    TokenTierView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        margin_text: format!("{:.2}", item.margin),
        quantity: item.quantity.to_string(),
        port: item.port.to_string(),
        weight: format!("{:.2}", item.weight),
        phone: text::truncate(&item.phone, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_tier_views(items: &[TokenTier]) -> Vec<TokenTierView> {
    let mut views: Vec<TokenTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_tier_view(item));
    }
    views
}

pub fn sum_token_tier_view(view: &TokenTierView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_margin + (parsed_quantity as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_tier("code-452", "title-367");
        let view = to_token_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
