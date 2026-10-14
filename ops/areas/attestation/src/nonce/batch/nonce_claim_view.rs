use super::*;

/// How screens and exports show a nonce claim.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceClaimView {
    pub code: String,
    pub label_text: String,
    pub contact_email: String,
    pub ttl_seconds: String,
    pub category: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_nonce_claim_view(item: &NonceClaim) -> NonceClaimView {
    NonceClaimView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        contact_email: item.contact_email.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        category: item.category.clone(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_claim_views(items: &[NonceClaim]) -> Vec<NonceClaimView> {
    let mut views: Vec<NonceClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_claim_view(item));
    }
    views
}

pub fn sum_nonce_claim_view(view: &NonceClaimView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_nonce_claim("code-856", "label-470");
        let view = to_nonce_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
