use super::*;

use acme_kit::text;

/// How screens and exports show a batched segment assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentAssignmentView {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub fingerprint_text: String,
    pub ratio_text: String,
    pub phone: String,
    pub reviewer: String,
    pub owner_id_text: String,
    pub status_label: String,
}

pub fn to_batched_segment_assignment_view(item: &BatchedSegmentAssignment) -> BatchedSegmentAssignmentView {
    BatchedSegmentAssignmentView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        currency: text::truncate(&item.currency, 40),
        fingerprint_text: item.fingerprint.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        phone: item.phone.clone(),
        reviewer: item.reviewer.clone(),
        owner_id_text: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_segment_assignment_views(items: &[BatchedSegmentAssignment]) -> Vec<BatchedSegmentAssignmentView> {
    let mut views: Vec<BatchedSegmentAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_segment_assignment_view(item));
    }
    views
}

pub fn sum_batched_segment_assignment_view(view: &BatchedSegmentAssignmentView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_segment_assignment("code-534", "name-575");
        let view = to_batched_segment_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_segment_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
