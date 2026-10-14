use super::*;

use acme_kit::text;

/// How screens and exports show a health check quota.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckQuotaView {
    pub code: String,
    pub title: String,
    pub budget: String,
    pub capacity: String,
    pub external_ref: String,
    pub scope_text: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_health_check_quota_view(item: &HealthCheckQuota) -> HealthCheckQuotaView {
    HealthCheckQuotaView {
        code: item.code.clone(),
        title: item.title.clone(),
        budget: format!("{:.2}", item.budget),
        capacity: item.capacity.to_string(),
        external_ref: text::truncate(&item.external_ref, 40),
        scope_text: item.scope.clone(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_quota_views(items: &[HealthCheckQuota]) -> Vec<HealthCheckQuotaView> {
    let mut views: Vec<HealthCheckQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_quota_view(item));
    }
    views
}

pub fn sum_health_check_quota_view(view: &HealthCheckQuotaView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_budget + (parsed_capacity as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_quota("code-334", "title-945");
        let view = to_health_check_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
