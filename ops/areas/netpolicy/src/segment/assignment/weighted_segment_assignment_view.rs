use super::*;

use acme_kit::text;

/// How screens and exports show a weighted segment assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentAssignmentView {
    pub code_text: String,
    pub label: String,
    pub weight: String,
    pub currency_text: String,
    pub budget_text: String,
    pub region_text: String,
    pub unit: String,
    pub phone: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_weighted_segment_assignment_view(item: &WeightedSegmentAssignment) -> WeightedSegmentAssignmentView {
    WeightedSegmentAssignmentView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        weight: format!("{:.2}", item.weight),
        currency_text: item.currency.clone(),
        budget_text: format!("{:.2}", item.budget),
        region_text: text::truncate(&item.region, 80),
        unit: item.unit.clone(),
        phone: text::truncate(&item.phone, 40),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_segment_assignment_views(items: &[WeightedSegmentAssignment]) -> Vec<WeightedSegmentAssignmentView> {
    let mut views: Vec<WeightedSegmentAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_segment_assignment_view(item));
    }
    views
}

pub fn sum_weighted_segment_assignment_view(view: &WeightedSegmentAssignmentView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_weight + parsed_budget + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_segment_assignment("code-297", "label-570");
        let view = to_weighted_segment_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_segment_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
