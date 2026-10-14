use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive silence assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceAssignmentView {
    pub code: String,
    pub title: String,
    pub tenant_id_text: String,
    pub updated_at_text: String,
    pub unit: String,
    pub cipher: String,
    pub category: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_adaptive_silence_assignment_view(item: &AdaptiveSilenceAssignment) -> AdaptiveSilenceAssignmentView {
    AdaptiveSilenceAssignmentView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 80),
        tenant_id_text: item.tenant_id.clone(),
        updated_at_text: item.updated_at.to_string(),
        unit: text::truncate(&item.unit, 40),
        cipher: text::truncate(&item.cipher, 64),
        category: item.category.clone(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_silence_assignment_views(items: &[AdaptiveSilenceAssignment]) -> Vec<AdaptiveSilenceAssignmentView> {
    let mut views: Vec<AdaptiveSilenceAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_silence_assignment_view(item));
    }
    views
}

pub fn sum_adaptive_silence_assignment_view(view: &AdaptiveSilenceAssignmentView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_silence_assignment("code-164", "title-380");
        let view = to_adaptive_silence_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_silence_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
