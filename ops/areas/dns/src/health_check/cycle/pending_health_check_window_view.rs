use super::*;

/// How screens and exports show a pending health check window.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckWindowView {
    pub code: String,
    pub name_text: String,
    pub limit: String,
    pub minimum: String,
    pub weight: String,
    pub locale_text: String,
    pub port_text: String,
    pub status_label: String,
}

pub fn to_pending_health_check_window_view(item: &PendingHealthCheckWindow) -> PendingHealthCheckWindowView {
    PendingHealthCheckWindowView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        limit: item.limit.to_string(),
        minimum: format!("{:.2}", item.minimum),
        weight: format!("{:.2}", item.weight),
        locale_text: item.locale.clone(),
        port_text: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_health_check_window_views(items: &[PendingHealthCheckWindow]) -> Vec<PendingHealthCheckWindowView> {
    let mut views: Vec<PendingHealthCheckWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_health_check_window_view(item));
    }
    views
}

pub fn sum_pending_health_check_window_view(view: &PendingHealthCheckWindowView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_limit as f64 + parsed_minimum + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_health_check_window("code-300", "name-530");
        let view = to_pending_health_check_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_health_check_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
