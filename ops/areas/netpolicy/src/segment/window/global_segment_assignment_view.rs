use super::*;

use acme_kit::text;

/// How screens and exports show a global segment assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentAssignmentView {
    pub code: String,
    pub name_text: String,
    pub description: String,
    pub starts_at_text: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_global_segment_assignment_view(item: &GlobalSegmentAssignment) -> GlobalSegmentAssignmentView {
    GlobalSegmentAssignmentView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        description: text::truncate(&item.description, 80),
        starts_at_text: item.starts_at.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_segment_assignment_views(items: &[GlobalSegmentAssignment]) -> Vec<GlobalSegmentAssignmentView> {
    let mut views: Vec<GlobalSegmentAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_segment_assignment_view(item));
    }
    views
}

pub fn sum_global_segment_assignment_view(view: &GlobalSegmentAssignmentView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_segment_assignment("code-3", "name-761");
        let view = to_global_segment_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_segment_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
