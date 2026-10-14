use super::*;

/// How screens and exports show a audit event plan.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPlanView {
    pub code: String,
    pub title: String,
    pub cost: String,
    pub channel: String,
    pub grace_days: String,
    pub owner_id_text: String,
    pub floor_value_text: String,
    pub retries: String,
    pub updated_at_text: String,
    pub rotation_days: String,
    pub quota_units_text: String,
    pub status_label: String,
}

pub fn to_audit_event_plan_view(item: &AuditEventPlan) -> AuditEventPlanView {
    AuditEventPlanView {
        code: item.code.clone(),
        title: item.title.clone(),
        cost: format!("{:.2}", item.cost),
        channel: item.channel.clone(),
        grace_days: item.grace_days.to_string(),
        owner_id_text: item.owner_id.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        retries: item.retries.to_string(),
        updated_at_text: item.updated_at.to_string(),
        rotation_days: item.rotation_days.to_string(),
        quota_units_text: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_plan_views(items: &[AuditEventPlan]) -> Vec<AuditEventPlanView> {
    let mut views: Vec<AuditEventPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_plan_view(item));
    }
    views
}

pub fn sum_audit_event_plan_view(view: &AuditEventPlanView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_cost + (parsed_grace_days as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_plan("code-319", "title-17");
        let view = to_audit_event_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
