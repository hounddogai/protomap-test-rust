use super::*;

use acme_kit::text;

/// How screens and exports show a capped page proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageProposalView {
    pub code: String,
    pub name: String,
    pub quantity_text: String,
    pub margin: String,
    pub scope: String,
    pub ceiling_text: String,
    pub timezone: String,
    pub checksum: String,
    pub contact_email: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_capped_page_proposal_view(item: &CappedPageProposal) -> CappedPageProposalView {
    CappedPageProposalView {
        code: item.code.clone(),
        name: item.name.clone(),
        quantity_text: item.quantity.to_string(),
        margin: format!("{:.2}", item.margin),
        scope: item.scope.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        timezone: item.timezone.clone(),
        checksum: text::truncate(&item.checksum, 40),
        contact_email: text::truncate(&item.contact_email, 80),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_page_proposal_views(items: &[CappedPageProposal]) -> Vec<CappedPageProposalView> {
    let mut views: Vec<CappedPageProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_page_proposal_view(item));
    }
    views
}

pub fn sum_capped_page_proposal_view(view: &CappedPageProposalView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_margin + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_page_proposal("code-358", "name-672");
        let view = to_capped_page_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_page_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
