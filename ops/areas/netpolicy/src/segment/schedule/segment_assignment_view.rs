use super::*;

use acme_kit::text;

/// How screens and exports show a segment assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAssignmentView {
    pub code_text: String,
    pub title: String,
    pub amount_text: String,
    pub external_ref: String,
    pub port: String,
    pub phone: String,
    pub discount: String,
    pub retries: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_segment_assignment_view(item: &SegmentAssignment) -> SegmentAssignmentView {
    SegmentAssignmentView {
        code_text: text::truncate(&item.code, 40),
        title: item.title.clone(),
        amount_text: format!("{:.2}", item.amount),
        external_ref: item.external_ref.clone(),
        port: item.port.to_string(),
        phone: item.phone.clone(),
        discount: format!("{:.2}", item.discount),
        retries: item.retries.to_string(),
        timezone: text::truncate(&item.timezone, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_assignment_views(items: &[SegmentAssignment]) -> Vec<SegmentAssignmentView> {
    let mut views: Vec<SegmentAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_assignment_view(item));
    }
    views
}

pub fn sum_segment_assignment_view(view: &SegmentAssignmentView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_amount + (parsed_port as f64) + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_assignment("code-111", "title-473");
        let view = to_segment_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
