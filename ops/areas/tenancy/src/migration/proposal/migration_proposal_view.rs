use super::*;

use acme_kit::text;

/// How screens and exports show a migration proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationProposalView {
    pub code: String,
    pub title: String,
    pub threshold_text: String,
    pub expires_at: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_migration_proposal_view(item: &MigrationProposal) -> MigrationProposalView {
    MigrationProposalView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        threshold_text: format!("{:.2}", item.threshold),
        expires_at: item.expires_at.to_string(),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_proposal_views(items: &[MigrationProposal]) -> Vec<MigrationProposalView> {
    let mut views: Vec<MigrationProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_proposal_view(item));
    }
    views
}

pub fn sum_migration_proposal_view(view: &MigrationProposalView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_threshold + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_proposal("code-433", "title-969");
        let view = to_migration_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
