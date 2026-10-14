use super::*;

use acme_kit::text;

/// How screens and exports show a page claim.
#[derive(Debug, Clone, PartialEq)]
pub struct PageClaimView {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub source_text: String,
    pub currency: String,
    pub created_at: String,
    pub fingerprint: String,
    pub weight: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_page_claim_view(item: &PageClaim) -> PageClaimView {
    PageClaimView {
        code: item.code.clone(),
        name: item.name.clone(),
        owner_id: item.owner_id.clone(),
        source_text: item.source.clone(),
        currency: item.currency.clone(),
        created_at: item.created_at.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 40),
        weight: format!("{:.2}", item.weight),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_claim_views(items: &[PageClaim]) -> Vec<PageClaimView> {
    let mut views: Vec<PageClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_claim_view(item));
    }
    views
}

pub fn sum_page_claim_view(view: &PageClaimView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_page_claim("code-90", "name-29");
        let view = to_page_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_page_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
