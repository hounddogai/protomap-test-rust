use super::*;

/// How screens and exports show a slo assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct SloAssignmentView {
    pub code: String,
    pub title: String,
    pub amount_text: String,
    pub segment_text: String,
    pub starts_at: String,
    pub reviewer_text: String,
    pub status_label: String,
}

pub fn to_slo_assignment_view(item: &SloAssignment) -> SloAssignmentView {
    SloAssignmentView {
        code: item.code.clone(),
        title: item.title.clone(),
        amount_text: format!("{:.2}", item.amount),
        segment_text: item.segment.clone(),
        starts_at: item.starts_at.to_string(),
        reviewer_text: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_assignment_views(items: &[SloAssignment]) -> Vec<SloAssignmentView> {
    let mut views: Vec<SloAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_assignment_view(item));
    }
    views
}

pub fn sum_slo_assignment_view(view: &SloAssignmentView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_amount + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_assignment("code-231", "title-369");
        let view = to_slo_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
