use super::*;

/// How screens and exports show a probe proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeProposalView {
    pub code: String,
    pub name: String,
    pub discount: String,
    pub key_id: String,
    pub timezone: String,
    pub country: String,
    pub maximum_text: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_probe_proposal_view(item: &ProbeProposal) -> ProbeProposalView {
    ProbeProposalView {
        code: item.code.clone(),
        name: item.name.clone(),
        discount: format!("{:.2}", item.discount),
        key_id: item.key_id.clone(),
        timezone: item.timezone.clone(),
        country: item.country.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_proposal_views(items: &[ProbeProposal]) -> Vec<ProbeProposalView> {
    let mut views: Vec<ProbeProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_proposal_view(item));
    }
    views
}

pub fn sum_probe_proposal_view(view: &ProbeProposalView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_discount + parsed_maximum + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_proposal("code-349", "name-601");
        let view = to_probe_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
