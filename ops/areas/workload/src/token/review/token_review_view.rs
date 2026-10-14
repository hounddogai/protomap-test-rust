use super::*;

use acme_kit::text;

/// How screens and exports show a token review.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenReviewView {
    pub code_text: String,
    pub name: String,
    pub ratio: String,
    pub namespace: String,
    pub principal_text: String,
    pub checksum_text: String,
    pub rate_text: String,
    pub scope_text: String,
    pub reviewer_text: String,
    pub quantity: String,
    pub expires_at_text: String,
    pub status_label: String,
}

pub fn to_token_review_view(item: &TokenReview) -> TokenReviewView {
    TokenReviewView {
        code_text: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 64),
        ratio: format!("{:.2}", item.ratio),
        namespace: text::truncate(&item.namespace, 64),
        principal_text: item.principal.clone(),
        checksum_text: text::truncate(&item.checksum, 80),
        rate_text: format!("{:.2}", item.rate),
        scope_text: item.scope.clone(),
        reviewer_text: text::truncate(&item.reviewer, 120),
        quantity: item.quantity.to_string(),
        expires_at_text: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_review_views(items: &[TokenReview]) -> Vec<TokenReviewView> {
    let mut views: Vec<TokenReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_review_view(item));
    }
    views
}

pub fn sum_token_review_view(view: &TokenReviewView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_ratio + parsed_rate + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_review("code-540", "name-41");
        let view = to_token_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
