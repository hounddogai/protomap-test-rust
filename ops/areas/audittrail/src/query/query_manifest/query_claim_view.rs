use super::*;

use acme_kit::text;

/// How screens and exports show a query claim.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryClaimView {
    pub code: String,
    pub title_text: String,
    pub summary: String,
    pub currency_text: String,
    pub updated_at_text: String,
    pub reviewer: String,
    pub checksum: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_query_claim_view(item: &QueryClaim) -> QueryClaimView {
    QueryClaimView {
        code: text::truncate(&item.code, 40),
        title_text: text::truncate(&item.title, 80),
        summary: item.summary.clone(),
        currency_text: item.currency.clone(),
        updated_at_text: item.updated_at.to_string(),
        reviewer: text::truncate(&item.reviewer, 80),
        checksum: item.checksum.clone(),
        scope: text::truncate(&item.scope, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_claim_views(items: &[QueryClaim]) -> Vec<QueryClaimView> {
    let mut views: Vec<QueryClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_claim_view(item));
    }
    views
}

pub fn sum_query_claim_view(view: &QueryClaimView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_updated_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_claim("code-62", "title-754");
        let view = to_query_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
