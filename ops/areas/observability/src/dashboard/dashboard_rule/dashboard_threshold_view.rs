use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardThresholdView {
    pub code: String,
    pub name: String,
    pub namespace_text: String,
    pub cluster_text: String,
    pub threshold_text: String,
    pub key_id: String,
    pub description_text: String,
    pub contact_email_text: String,
    pub limit: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_dashboard_threshold_view(item: &DashboardThreshold) -> DashboardThresholdView {
    DashboardThresholdView {
        code: item.code.clone(),
        name: item.name.clone(),
        namespace_text: item.namespace.clone(),
        cluster_text: item.cluster.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        key_id: text::truncate(&item.key_id, 80),
        description_text: text::truncate(&item.description, 120),
        contact_email_text: item.contact_email.clone(),
        limit: item.limit.to_string(),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_threshold_views(items: &[DashboardThreshold]) -> Vec<DashboardThresholdView> {
    let mut views: Vec<DashboardThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_threshold_view(item));
    }
    views
}

pub fn sum_dashboard_threshold_view(view: &DashboardThresholdView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_threshold + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_threshold("code-26", "name-153");
        let view = to_dashboard_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
