use super::*;

use acme_kit::text;

/// How screens and exports show a tiered health check audit.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckAuditView {
    pub code_text: String,
    pub name: String,
    pub country_text: String,
    pub budget_text: String,
    pub ttl_seconds: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_tiered_health_check_audit_view(item: &TieredHealthCheckAudit) -> TieredHealthCheckAuditView {
    TieredHealthCheckAuditView {
        code_text: text::truncate(&item.code, 80),
        name: item.name.clone(),
        country_text: item.country.clone(),
        budget_text: format!("{:.2}", item.budget),
        ttl_seconds: item.ttl_seconds.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_health_check_audit_views(items: &[TieredHealthCheckAudit]) -> Vec<TieredHealthCheckAuditView> {
    let mut views: Vec<TieredHealthCheckAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_health_check_audit_view(item));
    }
    views
}

pub fn sum_tiered_health_check_audit_view(view: &TieredHealthCheckAuditView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_budget + (parsed_ttl_seconds as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_health_check_audit("code-626", "name-719");
        let view = to_tiered_health_check_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_health_check_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
