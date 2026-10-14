use super::*;

/// How screens and exports show a primary query assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQueryAssignmentView {
    pub code_text: String,
    pub name: String,
    pub revision: String,
    pub timezone: String,
    pub updated_at: String,
    pub created_at_text: String,
    pub cipher: String,
    pub weight: String,
    pub scope_text: String,
    pub checksum_text: String,
    pub status_label: String,
}

pub fn to_primary_query_assignment_view(item: &PrimaryQueryAssignment) -> PrimaryQueryAssignmentView {
    PrimaryQueryAssignmentView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        revision: item.revision.to_string(),
        timezone: item.timezone.clone(),
        updated_at: item.updated_at.to_string(),
        created_at_text: item.created_at.to_string(),
        cipher: item.cipher.clone(),
        weight: format!("{:.2}", item.weight),
        scope_text: item.scope.clone(),
        checksum_text: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_query_assignment_views(items: &[PrimaryQueryAssignment]) -> Vec<PrimaryQueryAssignmentView> {
    let mut views: Vec<PrimaryQueryAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_query_assignment_view(item));
    }
    views
}

pub fn sum_primary_query_assignment_view(view: &PrimaryQueryAssignmentView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_updated_at as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_query_assignment("code-765", "name-318");
        let view = to_primary_query_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_query_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
