use super::*;

/// How screens and exports show a strict usage plan.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictUsagePlanView {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub minimum_text: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_strict_usage_plan_view(item: &StrictUsagePlan) -> StrictUsagePlanView {
    StrictUsagePlanView {
        code: item.code.clone(),
        name: item.name.clone(),
        namespace: item.namespace.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_usage_plan_views(items: &[StrictUsagePlan]) -> Vec<StrictUsagePlanView> {
    let mut views: Vec<StrictUsagePlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_usage_plan_view(item));
    }
    views
}

pub fn sum_strict_usage_plan_view(view: &StrictUsagePlanView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_minimum + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_usage_plan("code-31", "name-22");
        let view = to_strict_usage_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_usage_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
