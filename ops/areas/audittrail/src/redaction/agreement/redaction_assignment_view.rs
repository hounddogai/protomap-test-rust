use super::*;

use acme_kit::text;

/// How screens and exports show a redaction assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAssignmentView {
    pub code_text: String,
    pub title_text: String,
    pub cluster: String,
    pub attempts: String,
    pub rotation_days: String,
    pub issuer: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_redaction_assignment_view(item: &RedactionAssignment) -> RedactionAssignmentView {
    RedactionAssignmentView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        cluster: text::truncate(&item.cluster, 80),
        attempts: item.attempts.to_string(),
        rotation_days: item.rotation_days.to_string(),
        issuer: item.issuer.clone(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_assignment_views(items: &[RedactionAssignment]) -> Vec<RedactionAssignmentView> {
    let mut views: Vec<RedactionAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_assignment_view(item));
    }
    views
}

pub fn sum_redaction_assignment_view(view: &RedactionAssignmentView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_redaction_assignment("code-166", "title-50");
        let view = to_redaction_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_redaction_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
