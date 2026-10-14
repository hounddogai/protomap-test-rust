use super::*;

/// How screens and exports show a legacy binding proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingProposalView {
    pub code_text: String,
    pub title: String,
    pub reviewer: String,
    pub attempts: String,
    pub floor_value_text: String,
    pub status_label: String,
}

pub fn to_legacy_binding_proposal_view(item: &LegacyBindingProposal) -> LegacyBindingProposalView {
    LegacyBindingProposalView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        reviewer: item.reviewer.clone(),
        attempts: item.attempts.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_binding_proposal_views(items: &[LegacyBindingProposal]) -> Vec<LegacyBindingProposalView> {
    let mut views: Vec<LegacyBindingProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_binding_proposal_view(item));
    }
    views
}

pub fn sum_legacy_binding_proposal_view(view: &LegacyBindingProposalView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_binding_proposal("code-399", "title-202");
        let view = to_legacy_binding_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_binding_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
