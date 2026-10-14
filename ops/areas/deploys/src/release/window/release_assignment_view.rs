use super::*;

use acme_kit::text;

/// How screens and exports show a release assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAssignmentView {
    pub code: String,
    pub name: String,
    pub active_text: String,
    pub description: String,
    pub maximum: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_release_assignment_view(item: &ReleaseAssignment) -> ReleaseAssignmentView {
    ReleaseAssignmentView {
        code: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 40),
        active_text: format!("{}", item.active),
        description: text::truncate(&item.description, 120),
        maximum: format!("{:.2}", item.maximum),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_assignment_views(items: &[ReleaseAssignment]) -> Vec<ReleaseAssignmentView> {
    let mut views: Vec<ReleaseAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_assignment_view(item));
    }
    views
}

pub fn sum_release_assignment_view(view: &ReleaseAssignmentView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_assignment("code-615", "name-755");
        let view = to_release_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
