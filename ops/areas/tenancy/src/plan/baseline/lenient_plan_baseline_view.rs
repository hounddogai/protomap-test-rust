use super::*;

use acme_kit::text;

/// How screens and exports show a lenient plan baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanBaselineView {
    pub code: String,
    pub label: String,
    pub attempts_text: String,
    pub timezone: String,
    pub visible_text: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_lenient_plan_baseline_view(item: &LenientPlanBaseline) -> LenientPlanBaselineView {
    LenientPlanBaselineView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        attempts_text: item.attempts.to_string(),
        timezone: item.timezone.clone(),
        visible_text: format!("{}", item.visible),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_plan_baseline_views(items: &[LenientPlanBaseline]) -> Vec<LenientPlanBaselineView> {
    let mut views: Vec<LenientPlanBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_plan_baseline_view(item));
    }
    views
}

pub fn sum_lenient_plan_baseline_view(view: &LenientPlanBaselineView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_plan_baseline("code-539", "label-917");
        let view = to_lenient_plan_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_plan_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
