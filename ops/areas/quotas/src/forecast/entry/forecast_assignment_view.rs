use super::*;

use acme_kit::text;

/// How screens and exports show a forecast assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAssignmentView {
    pub code: String,
    pub label_text: String,
    pub source_text: String,
    pub rotation_days: String,
    pub quantity_text: String,
    pub ttl_seconds_text: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_forecast_assignment_view(item: &ForecastAssignment) -> ForecastAssignmentView {
    ForecastAssignmentView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        source_text: text::truncate(&item.source, 120),
        rotation_days: item.rotation_days.to_string(),
        quantity_text: item.quantity.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        scope: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_assignment_views(items: &[ForecastAssignment]) -> Vec<ForecastAssignmentView> {
    let mut views: Vec<ForecastAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_assignment_view(item));
    }
    views
}

pub fn sum_forecast_assignment_view(view: &ForecastAssignmentView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_quantity as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_assignment("code-470", "label-867");
        let view = to_forecast_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
