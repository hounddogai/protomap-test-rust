use super::*;

use acme_kit::text;

/// How screens and exports show a global tenant bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTenantBundleView {
    pub code_text: String,
    pub name: String,
    pub unit: String,
    pub cluster: String,
    pub weight: String,
    pub starts_at: String,
    pub active: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_global_tenant_bundle_view(item: &GlobalTenantBundle) -> GlobalTenantBundleView {
    GlobalTenantBundleView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        unit: text::truncate(&item.unit, 120),
        cluster: text::truncate(&item.cluster, 40),
        weight: format!("{:.2}", item.weight),
        starts_at: item.starts_at.to_string(),
        active: format!("{}", item.active),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_tenant_bundle_views(items: &[GlobalTenantBundle]) -> Vec<GlobalTenantBundleView> {
    let mut views: Vec<GlobalTenantBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_tenant_bundle_view(item));
    }
    views
}

pub fn sum_global_tenant_bundle_view(view: &GlobalTenantBundleView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_weight + (parsed_starts_at as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_tenant_bundle("code-648", "name-819");
        let view = to_global_tenant_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_tenant_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
