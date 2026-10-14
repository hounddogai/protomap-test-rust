use super::*;

use acme_kit::text;

/// How screens and exports show a usage bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageBundleView {
    pub code: String,
    pub name_text: String,
    pub audience_text: String,
    pub external_ref: String,
    pub tenant_id: String,
    pub region: String,
    pub quantity: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_usage_bundle_view(item: &UsageBundle) -> UsageBundleView {
    UsageBundleView {
        code: text::truncate(&item.code, 80),
        name_text: text::truncate(&item.name, 80),
        audience_text: text::truncate(&item.audience, 40),
        external_ref: item.external_ref.clone(),
        tenant_id: item.tenant_id.clone(),
        region: item.region.clone(),
        quantity: item.quantity.to_string(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_bundle_views(items: &[UsageBundle]) -> Vec<UsageBundleView> {
    let mut views: Vec<UsageBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_bundle_view(item));
    }
    views
}

pub fn sum_usage_bundle_view(view: &UsageBundleView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_bundle("code-817", "name-115");
        let view = to_usage_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
