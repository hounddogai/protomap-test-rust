use super::*;

/// How screens and exports show a egress proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressProposalView {
    pub code_text: String,
    pub name: String,
    pub title: String,
    pub floor_value_text: String,
    pub audience_text: String,
    pub starts_at: String,
    pub port_text: String,
    pub ratio: String,
    pub visible: String,
    pub tenant_id_text: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_egress_proposal_view(item: &EgressProposal) -> EgressProposalView {
    EgressProposalView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        title: item.title.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        audience_text: item.audience.clone(),
        starts_at: item.starts_at.to_string(),
        port_text: item.port.to_string(),
        ratio: format!("{:.2}", item.ratio),
        visible: format!("{}", item.visible),
        tenant_id_text: item.tenant_id.clone(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_proposal_views(items: &[EgressProposal]) -> Vec<EgressProposalView> {
    let mut views: Vec<EgressProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_proposal_view(item));
    }
    views
}

pub fn sum_egress_proposal_view(view: &EgressProposalView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_floor_value + (parsed_starts_at as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_proposal("code-750", "name-283");
        let view = to_egress_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
