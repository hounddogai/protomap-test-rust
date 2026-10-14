use super::*;

use acme_kit::text;

/// How screens and exports show a fallback rollout plan.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRolloutPlanView {
    pub code_text: String,
    pub name: String,
    pub margin: String,
    pub external_ref: String,
    pub rate_text: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_fallback_rollout_plan_view(item: &FallbackRolloutPlan) -> FallbackRolloutPlanView {
    FallbackRolloutPlanView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        margin: format!("{:.2}", item.margin),
        external_ref: text::truncate(&item.external_ref, 40),
        rate_text: format!("{:.2}", item.rate),
        segment: text::truncate(&item.segment, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_rollout_plan_views(items: &[FallbackRolloutPlan]) -> Vec<FallbackRolloutPlanView> {
    let mut views: Vec<FallbackRolloutPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_rollout_plan_view(item));
    }
    views
}

pub fn sum_fallback_rollout_plan_view(view: &FallbackRolloutPlanView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_margin + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_rollout_plan("code-171", "name-139");
        let view = to_fallback_rollout_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_rollout_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
