use super::*;

use acme_kit::text;

/// How screens and exports show a tenant metric.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantMetricView {
    pub code: String,
    pub label_text: String,
    pub scope: String,
    pub created_at_text: String,
    pub budget: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_tenant_metric_view(item: &TenantMetric) -> TenantMetricView {
    TenantMetricView {
        code: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        scope: item.scope.clone(),
        created_at_text: item.created_at.to_string(),
        budget: format!("{:.2}", item.budget),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_metric_views(items: &[TenantMetric]) -> Vec<TenantMetricView> {
    let mut views: Vec<TenantMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_metric_view(item));
    }
    views
}

pub fn sum_tenant_metric_view(view: &TenantMetricView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_metric("code-343", "label-838");
        let view = to_tenant_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
