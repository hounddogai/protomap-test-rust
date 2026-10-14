use super::*;

/// How screens and exports show a plan policy.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanPolicyView {
    pub code: String,
    pub name: String,
    pub version_text: String,
    pub active: String,
    pub phone: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_plan_policy_view(item: &PlanPolicy) -> PlanPolicyView {
    PlanPolicyView {
        code: item.code.clone(),
        name: item.name.clone(),
        version_text: item.version.to_string(),
        active: format!("{}", item.active),
        phone: item.phone.clone(),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_policy_views(items: &[PlanPolicy]) -> Vec<PlanPolicyView> {
    let mut views: Vec<PlanPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_policy_view(item));
    }
    views
}

pub fn sum_plan_policy_view(view: &PlanPolicyView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_version as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_policy("code-951", "name-360");
        let view = to_plan_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
