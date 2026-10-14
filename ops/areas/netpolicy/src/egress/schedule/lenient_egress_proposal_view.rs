use super::*;

use acme_kit::text;

/// How screens and exports show a lenient egress proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressProposalView {
    pub code: String,
    pub name_text: String,
    pub port: String,
    pub floor_value: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_lenient_egress_proposal_view(item: &LenientEgressProposal) -> LenientEgressProposalView {
    LenientEgressProposalView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        port: item.port.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        phone: text::truncate(&item.phone, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_egress_proposal_views(items: &[LenientEgressProposal]) -> Vec<LenientEgressProposalView> {
    let mut views: Vec<LenientEgressProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_egress_proposal_view(item));
    }
    views
}

pub fn sum_lenient_egress_proposal_view(view: &LenientEgressProposalView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_port as f64 + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_egress_proposal("code-425", "name-862");
        let view = to_lenient_egress_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_egress_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
