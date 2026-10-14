use super::*;

/// How screens and exports show a binding plan.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingPlanView {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub cost: String,
    pub batch_size: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_binding_plan_view(item: &BindingPlan) -> BindingPlanView {
    BindingPlanView {
        code: item.code.clone(),
        name: item.name.clone(),
        fingerprint: item.fingerprint.clone(),
        cost: format!("{:.2}", item.cost),
        batch_size: item.batch_size.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_plan_views(items: &[BindingPlan]) -> Vec<BindingPlanView> {
    let mut views: Vec<BindingPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_plan_view(item));
    }
    views
}

pub fn sum_binding_plan_view(view: &BindingPlanView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_cost + (parsed_batch_size as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_binding_plan("code-269", "name-172");
        let view = to_binding_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_binding_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
