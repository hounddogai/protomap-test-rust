use super::*;

/// How screens and exports show a tenant plan.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantPlanView {
    pub code: String,
    pub label_text: String,
    pub ceiling: String,
    pub created_at: String,
    pub summary: String,
    pub cluster_text: String,
    pub grace_days: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_tenant_plan_view(item: &TenantPlan) -> TenantPlanView {
    TenantPlanView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        created_at: item.created_at.to_string(),
        summary: item.summary.clone(),
        cluster_text: item.cluster.clone(),
        grace_days: item.grace_days.to_string(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_plan_views(items: &[TenantPlan]) -> Vec<TenantPlanView> {
    let mut views: Vec<TenantPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_plan_view(item));
    }
    views
}

pub fn sum_tenant_plan_view(view: &TenantPlanView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_ceiling + (parsed_created_at as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_plan("code-254", "label-845");
        let view = to_tenant_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
