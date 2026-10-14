use super::*;

use acme_kit::text;

/// How screens and exports show a global freeze proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFreezeProposalView {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub grace_days_text: String,
    pub audience: String,
    pub approved: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_global_freeze_proposal_view(item: &GlobalFreezeProposal) -> GlobalFreezeProposalView {
    GlobalFreezeProposalView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        contact_email: item.contact_email.clone(),
        grace_days_text: item.grace_days.to_string(),
        audience: item.audience.clone(),
        approved: format!("{}", item.approved),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_freeze_proposal_views(items: &[GlobalFreezeProposal]) -> Vec<GlobalFreezeProposalView> {
    let mut views: Vec<GlobalFreezeProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_freeze_proposal_view(item));
    }
    views
}

pub fn sum_global_freeze_proposal_view(view: &GlobalFreezeProposalView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_freeze_proposal("code-39", "name-568");
        let view = to_global_freeze_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_freeze_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
