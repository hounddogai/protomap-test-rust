use super::*;

/// How screens and exports show a allowlist metric.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMetricView {
    pub code: String,
    pub name: String,
    pub floor_value: String,
    pub expires_at: String,
    pub rate: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_allowlist_metric_view(item: &AllowlistMetric) -> AllowlistMetricView {
    AllowlistMetricView {
        code: item.code.clone(),
        name: item.name.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        expires_at: item.expires_at.to_string(),
        rate: format!("{:.2}", item.rate),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_metric_views(items: &[AllowlistMetric]) -> Vec<AllowlistMetricView> {
    let mut views: Vec<AllowlistMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_metric_view(item));
    }
    views
}

pub fn sum_allowlist_metric_view(view: &AllowlistMetricView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
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
    Ok(parsed_floor_value + (parsed_expires_at as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_allowlist_metric("code-296", "name-282");
        let view = to_allowlist_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_allowlist_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
