use super::*;

/// How screens and exports show a scheduled health check audit.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckAuditView {
    pub code_text: String,
    pub name_text: String,
    pub weight: String,
    pub revision_text: String,
    pub quantity_text: String,
    pub capacity: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_scheduled_health_check_audit_view(item: &ScheduledHealthCheckAudit) -> ScheduledHealthCheckAuditView {
    ScheduledHealthCheckAuditView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        weight: format!("{:.2}", item.weight),
        revision_text: item.revision.to_string(),
        quantity_text: item.quantity.to_string(),
        capacity: item.capacity.to_string(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_health_check_audit_views(
    items: &[ScheduledHealthCheckAudit],
) -> Vec<ScheduledHealthCheckAuditView> {
    let mut views: Vec<ScheduledHealthCheckAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_health_check_audit_view(item));
    }
    views
}

pub fn sum_scheduled_health_check_audit_view(view: &ScheduledHealthCheckAuditView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_weight + (parsed_revision as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_health_check_audit("code-941", "name-872");
        let view = to_scheduled_health_check_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_health_check_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
