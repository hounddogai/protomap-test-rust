use super::*;

use acme_kit::text;

/// How screens and exports show a health check group.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckGroupView {
    pub code: String,
    pub name: String,
    pub approved: String,
    pub timezone: String,
    pub principal_text: String,
    pub port: String,
    pub rate: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_health_check_group_view(item: &HealthCheckGroup) -> HealthCheckGroupView {
    HealthCheckGroupView {
        code: item.code.clone(),
        name: item.name.clone(),
        approved: format!("{}", item.approved),
        timezone: text::truncate(&item.timezone, 80),
        principal_text: item.principal.clone(),
        port: item.port.to_string(),
        rate: format!("{:.2}", item.rate),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_group_views(items: &[HealthCheckGroup]) -> Vec<HealthCheckGroupView> {
    let mut views: Vec<HealthCheckGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_group_view(item));
    }
    views
}

pub fn sum_health_check_group_view(view: &HealthCheckGroupView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_port as f64 + parsed_rate + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_group("code-325", "name-231");
        let view = to_health_check_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
