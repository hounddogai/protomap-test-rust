use super::*;

use acme_kit::text;

/// How screens and exports show a tiered allowlist claim.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistClaimView {
    pub code_text: String,
    pub title_text: String,
    pub tenant_id_text: String,
    pub updated_at_text: String,
    pub pinned: String,
    pub category: String,
    pub ceiling: String,
    pub expires_at: String,
    pub score: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_tiered_allowlist_claim_view(item: &TieredAllowlistClaim) -> TieredAllowlistClaimView {
    TieredAllowlistClaimView {
        code_text: text::truncate(&item.code, 80),
        title_text: text::truncate(&item.title, 80),
        tenant_id_text: item.tenant_id.clone(),
        updated_at_text: item.updated_at.to_string(),
        pinned: format!("{}", item.pinned),
        category: item.category.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        expires_at: item.expires_at.to_string(),
        score: format!("{:.2}", item.score),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_allowlist_claim_views(items: &[TieredAllowlistClaim]) -> Vec<TieredAllowlistClaimView> {
    let mut views: Vec<TieredAllowlistClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_allowlist_claim_view(item));
    }
    views
}

pub fn sum_tiered_allowlist_claim_view(view: &TieredAllowlistClaimView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_ceiling + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_allowlist_claim("code-214", "title-677");
        let view = to_tiered_allowlist_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_allowlist_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
