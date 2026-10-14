use super::*;

use acme_kit::text;

/// How screens and exports show a nonce proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceProposalView {
    pub code_text: String,
    pub title_text: String,
    pub visible: String,
    pub margin: String,
    pub summary: String,
    pub cost: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_nonce_proposal_view(item: &NonceProposal) -> NonceProposalView {
    NonceProposalView {
        code_text: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        visible: format!("{}", item.visible),
        margin: format!("{:.2}", item.margin),
        summary: item.summary.clone(),
        cost: format!("{:.2}", item.cost),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_proposal_views(items: &[NonceProposal]) -> Vec<NonceProposalView> {
    let mut views: Vec<NonceProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_proposal_view(item));
    }
    views
}

pub fn sum_nonce_proposal_view(view: &NonceProposalView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_margin + parsed_cost + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_nonce_proposal("code-399", "title-230");
        let view = to_nonce_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
