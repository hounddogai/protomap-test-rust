use super::*;

use acme_kit::text;

/// How screens and exports show a forecast milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMilestoneView {
    pub code: String,
    pub label: String,
    pub score: String,
    pub external_ref: String,
    pub reviewer: String,
    pub discount_text: String,
    pub status_label: String,
}

pub fn to_forecast_milestone_view(item: &ForecastMilestone) -> ForecastMilestoneView {
    ForecastMilestoneView {
        code: item.code.clone(),
        label: item.label.clone(),
        score: format!("{:.2}", item.score),
        external_ref: text::truncate(&item.external_ref, 40),
        reviewer: item.reviewer.clone(),
        discount_text: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_milestone_views(items: &[ForecastMilestone]) -> Vec<ForecastMilestoneView> {
    let mut views: Vec<ForecastMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_milestone_view(item));
    }
    views
}

pub fn sum_forecast_milestone_view(view: &ForecastMilestoneView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_score + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_milestone("code-274", "label-250");
        let view = to_forecast_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
