use super::*;

use acme_kit::text;

/// How screens and exports show a canary proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryProposalView {
    pub code: String,
    pub label_text: String,
    pub minimum_text: String,
    pub locale: String,
    pub ratio_text: String,
    pub issuer: String,
    pub approved_text: String,
    pub status_label: String,
}

pub fn to_canary_proposal_view(item: &CanaryProposal) -> CanaryProposalView {
    CanaryProposalView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        locale: item.locale.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        issuer: text::truncate(&item.issuer, 40),
        approved_text: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_proposal_views(items: &[CanaryProposal]) -> Vec<CanaryProposalView> {
    let mut views: Vec<CanaryProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_proposal_view(item));
    }
    views
}

pub fn sum_canary_proposal_view(view: &CanaryProposalView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_minimum + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_proposal("code-733", "label-244");
        let view = to_canary_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
