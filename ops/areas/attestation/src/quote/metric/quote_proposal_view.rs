use super::*;

use acme_kit::text;

/// How screens and exports show a quote proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteProposalView {
    pub code: String,
    pub name_text: String,
    pub title_text: String,
    pub budget: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_quote_proposal_view(item: &QuoteProposal) -> QuoteProposalView {
    QuoteProposalView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        title_text: item.title.clone(),
        budget: format!("{:.2}", item.budget),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_proposal_views(items: &[QuoteProposal]) -> Vec<QuoteProposalView> {
    let mut views: Vec<QuoteProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_proposal_view(item));
    }
    views
}

pub fn sum_quote_proposal_view(view: &QuoteProposalView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_budget + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_proposal("code-787", "name-389");
        let view = to_quote_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
