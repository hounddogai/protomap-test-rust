use super::*;

use acme_kit::text;

/// How screens and exports show a lenient plan cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanCycleView {
    pub code: String,
    pub title_text: String,
    pub retries: String,
    pub threshold: String,
    pub locale: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_lenient_plan_cycle_view(item: &LenientPlanCycle) -> LenientPlanCycleView {
    LenientPlanCycleView {
        code: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        retries: item.retries.to_string(),
        threshold: format!("{:.2}", item.threshold),
        locale: text::truncate(&item.locale, 40),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_plan_cycle_views(items: &[LenientPlanCycle]) -> Vec<LenientPlanCycleView> {
    let mut views: Vec<LenientPlanCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_plan_cycle_view(item));
    }
    views
}

pub fn sum_lenient_plan_cycle_view(view: &LenientPlanCycleView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_retries as f64 + parsed_threshold + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_plan_cycle("code-791", "title-357");
        let view = to_lenient_plan_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_plan_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
