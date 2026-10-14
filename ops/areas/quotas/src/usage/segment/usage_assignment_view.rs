use super::*;

use acme_kit::text;

/// How screens and exports show a usage assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageAssignmentView {
    pub code: String,
    pub name: String,
    pub batch_size: String,
    pub key_id_text: String,
    pub timezone_text: String,
    pub visible_text: String,
    pub status_label: String,
}

pub fn to_usage_assignment_view(item: &UsageAssignment) -> UsageAssignmentView {
    UsageAssignmentView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        batch_size: item.batch_size.to_string(),
        key_id_text: item.key_id.clone(),
        timezone_text: item.timezone.clone(),
        visible_text: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_assignment_views(items: &[UsageAssignment]) -> Vec<UsageAssignmentView> {
    let mut views: Vec<UsageAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_assignment_view(item));
    }
    views
}

pub fn sum_usage_assignment_view(view: &UsageAssignmentView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_assignment("code-730", "name-161");
        let view = to_usage_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
