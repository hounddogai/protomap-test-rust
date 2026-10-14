use super::*;

use acme_kit::text;

/// How screens and exports show a pending health check group.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckGroupView {
    pub code: String,
    pub name: String,
    pub weight: String,
    pub expires_at: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_pending_health_check_group_view(item: &PendingHealthCheckGroup) -> PendingHealthCheckGroupView {
    PendingHealthCheckGroupView {
        code: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 40),
        weight: format!("{:.2}", item.weight),
        expires_at: item.expires_at.to_string(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_health_check_group_views(items: &[PendingHealthCheckGroup]) -> Vec<PendingHealthCheckGroupView> {
    let mut views: Vec<PendingHealthCheckGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_health_check_group_view(item));
    }
    views
}

pub fn sum_pending_health_check_group_view(view: &PendingHealthCheckGroupView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_weight + (parsed_expires_at as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_health_check_group("code-964", "name-678");
        let view = to_pending_health_check_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_health_check_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
