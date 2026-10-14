use super::*;

use acme_kit::text;

/// How screens and exports show a migration segment.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSegmentView {
    pub code_text: String,
    pub title: String,
    pub sequence: String,
    pub budget: String,
    pub principal: String,
    pub issuer: String,
    pub expires_at: String,
    pub ceiling_text: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_migration_segment_view(item: &MigrationSegment) -> MigrationSegmentView {
    MigrationSegmentView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        sequence: item.sequence.to_string(),
        budget: format!("{:.2}", item.budget),
        principal: item.principal.clone(),
        issuer: text::truncate(&item.issuer, 40),
        expires_at: item.expires_at.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_segment_views(items: &[MigrationSegment]) -> Vec<MigrationSegmentView> {
    let mut views: Vec<MigrationSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_segment_view(item));
    }
    views
}

pub fn sum_migration_segment_view(view: &MigrationSegmentView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_budget + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_segment("code-155", "title-271");
        let view = to_migration_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
