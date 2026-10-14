use super::*;

use acme_kit::text;

/// How screens and exports show a default forecast assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastAssignmentView {
    pub code: String,
    pub title: String,
    pub reviewer_text: String,
    pub active_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_default_forecast_assignment_view(item: &DefaultForecastAssignment) -> DefaultForecastAssignmentView {
    DefaultForecastAssignmentView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        reviewer_text: text::truncate(&item.reviewer, 80),
        active_text: format!("{}", item.active),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_forecast_assignment_views(items: &[DefaultForecastAssignment]) -> Vec<DefaultForecastAssignmentView> {
    let mut views: Vec<DefaultForecastAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_forecast_assignment_view(item));
    }
    views
}

pub fn sum_default_forecast_assignment_view(view: &DefaultForecastAssignmentView) -> Result<f64, String> {
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
        let item = new_default_forecast_assignment("code-238", "title-401");
        let view = to_default_forecast_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_forecast_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
