use super::*;

use acme_kit::text;

/// How screens and exports show a pending nonce claim.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceClaimView {
    pub code_text: String,
    pub name_text: String,
    pub cipher_text: String,
    pub tenant_id: String,
    pub source: String,
    pub issuer: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_pending_nonce_claim_view(item: &PendingNonceClaim) -> PendingNonceClaimView {
    PendingNonceClaimView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        cipher_text: item.cipher.clone(),
        tenant_id: item.tenant_id.clone(),
        source: item.source.clone(),
        issuer: text::truncate(&item.issuer, 120),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_nonce_claim_views(items: &[PendingNonceClaim]) -> Vec<PendingNonceClaimView> {
    let mut views: Vec<PendingNonceClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_nonce_claim_view(item));
    }
    views
}

pub fn sum_pending_nonce_claim_view(view: &PendingNonceClaimView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_nonce_claim("code-930", "name-938");
        let view = to_pending_nonce_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_nonce_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
