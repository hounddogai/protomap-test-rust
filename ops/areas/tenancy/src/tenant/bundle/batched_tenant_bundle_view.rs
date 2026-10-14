use super::*;

use acme_kit::text;

/// How screens and exports show a batched tenant bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantBundleView {
    pub code: String,
    pub name_text: String,
    pub namespace: String,
    pub region: String,
    pub description_text: String,
    pub capacity: String,
    pub currency_text: String,
    pub threshold: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_batched_tenant_bundle_view(item: &BatchedTenantBundle) -> BatchedTenantBundleView {
    BatchedTenantBundleView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        namespace: item.namespace.clone(),
        region: item.region.clone(),
        description_text: item.description.clone(),
        capacity: item.capacity.to_string(),
        currency_text: text::truncate(&item.currency, 120),
        threshold: format!("{:.2}", item.threshold),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_tenant_bundle_views(items: &[BatchedTenantBundle]) -> Vec<BatchedTenantBundleView> {
    let mut views: Vec<BatchedTenantBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_tenant_bundle_view(item));
    }
    views
}

pub fn sum_batched_tenant_bundle_view(view: &BatchedTenantBundleView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_threshold + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_tenant_bundle("code-820", "name-534");
        let view = to_batched_tenant_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_tenant_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
