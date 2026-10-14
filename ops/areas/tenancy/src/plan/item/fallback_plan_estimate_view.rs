use super::*;

use acme_kit::text;

/// How screens and exports show a fallback plan estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanEstimateView {
    pub code: String,
    pub name: String,
    pub window_days_text: String,
    pub notes: String,
    pub sequence: String,
    pub retries_text: String,
    pub status_label: String,
}

pub fn to_fallback_plan_estimate_view(item: &FallbackPlanEstimate) -> FallbackPlanEstimateView {
    FallbackPlanEstimateView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        window_days_text: item.window_days.to_string(),
        notes: text::truncate(&item.notes, 80),
        sequence: item.sequence.to_string(),
        retries_text: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_plan_estimate_views(items: &[FallbackPlanEstimate]) -> Vec<FallbackPlanEstimateView> {
    let mut views: Vec<FallbackPlanEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_plan_estimate_view(item));
    }
    views
}

pub fn sum_fallback_plan_estimate_view(view: &FallbackPlanEstimateView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_sequence as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_plan_estimate("code-803", "name-450");
        let view = to_fallback_plan_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_plan_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
