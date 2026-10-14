use super::*;

use acme_kit::text;

/// How screens and exports show a rolling canary assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryAssignmentView {
    pub code: String,
    pub label_text: String,
    pub category: String,
    pub visible: String,
    pub scope_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_rolling_canary_assignment_view(item: &RollingCanaryAssignment) -> RollingCanaryAssignmentView {
    RollingCanaryAssignmentView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        category: text::truncate(&item.category, 64),
        visible: format!("{}", item.visible),
        scope_text: item.scope.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_canary_assignment_views(items: &[RollingCanaryAssignment]) -> Vec<RollingCanaryAssignmentView> {
    let mut views: Vec<RollingCanaryAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_canary_assignment_view(item));
    }
    views
}

pub fn sum_rolling_canary_assignment_view(view: &RollingCanaryAssignmentView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_canary_assignment("code-524", "label-306");
        let view = to_rolling_canary_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_canary_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
