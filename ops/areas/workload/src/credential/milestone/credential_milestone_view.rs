use super::*;

/// How screens and exports show a credential milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialMilestoneView {
    pub code_text: String,
    pub name: String,
    pub score: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_credential_milestone_view(item: &CredentialMilestone) -> CredentialMilestoneView {
    CredentialMilestoneView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        score: format!("{:.2}", item.score),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_milestone_views(items: &[CredentialMilestone]) -> Vec<CredentialMilestoneView> {
    let mut views: Vec<CredentialMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_milestone_view(item));
    }
    views
}

pub fn sum_credential_milestone_view(view: &CredentialMilestoneView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_milestone("code-698", "name-171");
        let view = to_credential_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
