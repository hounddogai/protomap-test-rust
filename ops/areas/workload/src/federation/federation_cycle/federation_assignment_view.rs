use super::*;

/// How screens and exports show a federation assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationAssignmentView {
    pub code: String,
    pub title: String,
    pub capacity: String,
    pub updated_at_text: String,
    pub principal_text: String,
    pub maximum: String,
    pub created_at: String,
    pub retries: String,
    pub duration_minutes: String,
    pub scope: String,
    pub notes_text: String,
    pub status_label: String,
}

pub fn to_federation_assignment_view(item: &FederationAssignment) -> FederationAssignmentView {
    FederationAssignmentView {
        code: item.code.clone(),
        title: item.title.clone(),
        capacity: item.capacity.to_string(),
        updated_at_text: item.updated_at.to_string(),
        principal_text: item.principal.clone(),
        maximum: format!("{:.2}", item.maximum),
        created_at: item.created_at.to_string(),
        retries: item.retries.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        scope: item.scope.clone(),
        notes_text: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_assignment_views(items: &[FederationAssignment]) -> Vec<FederationAssignmentView> {
    let mut views: Vec<FederationAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_assignment_view(item));
    }
    views
}

pub fn sum_federation_assignment_view(view: &FederationAssignmentView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_updated_at as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_assignment("code-90", "title-613");
        let view = to_federation_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
