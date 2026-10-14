use super::*;

use acme_kit::text;

/// How screens and exports show a token proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenProposalView {
    pub code: String,
    pub name_text: String,
    pub quota_units: String,
    pub retries_text: String,
    pub issuer: String,
    pub grace_days: String,
    pub quantity: String,
    pub title: String,
    pub floor_value_text: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_token_proposal_view(item: &TokenProposal) -> TokenProposalView {
    TokenProposalView {
        code: text::truncate(&item.code, 64),
        name_text: item.name.clone(),
        quota_units: item.quota_units.to_string(),
        retries_text: item.retries.to_string(),
        issuer: text::truncate(&item.issuer, 120),
        grace_days: item.grace_days.to_string(),
        quantity: item.quantity.to_string(),
        title: text::truncate(&item.title, 64),
        floor_value_text: format!("{:.2}", item.floor_value),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_proposal_views(items: &[TokenProposal]) -> Vec<TokenProposalView> {
    let mut views: Vec<TokenProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_proposal_view(item));
    }
    views
}

pub fn sum_token_proposal_view(view: &TokenProposalView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_retries as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_proposal("code-660", "name-797");
        let view = to_token_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
