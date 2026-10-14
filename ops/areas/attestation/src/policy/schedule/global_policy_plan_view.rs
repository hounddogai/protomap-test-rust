use super::*;

/// How screens and exports show a global policy plan.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicyPlanView {
    pub code: String,
    pub name: String,
    pub score: String,
    pub cluster: String,
    pub ttl_seconds: String,
    pub port: String,
    pub floor_value_text: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_global_policy_plan_view(item: &GlobalPolicyPlan) -> GlobalPolicyPlanView {
    GlobalPolicyPlanView {
        code: item.code.clone(),
        name: item.name.clone(),
        score: format!("{:.2}", item.score),
        cluster: item.cluster.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        port: item.port.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_policy_plan_views(items: &[GlobalPolicyPlan]) -> Vec<GlobalPolicyPlanView> {
    let mut views: Vec<GlobalPolicyPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_policy_plan_view(item));
    }
    views
}

pub fn sum_global_policy_plan_view(view: &GlobalPolicyPlanView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_score + (parsed_ttl_seconds as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_policy_plan("code-878", "name-442");
        let view = to_global_policy_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_policy_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
