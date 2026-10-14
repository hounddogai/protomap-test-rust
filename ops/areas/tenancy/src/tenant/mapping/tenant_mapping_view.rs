use super::*;

use acme_kit::text;

/// How screens and exports show a tenant mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantMappingView {
    pub code: String,
    pub label: String,
    pub window_days_text: String,
    pub external_ref_text: String,
    pub ends_at: String,
    pub segment_text: String,
    pub expires_at: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_tenant_mapping_view(item: &TenantMapping) -> TenantMappingView {
    TenantMappingView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        window_days_text: item.window_days.to_string(),
        external_ref_text: item.external_ref.clone(),
        ends_at: item.ends_at.to_string(),
        segment_text: text::truncate(&item.segment, 120),
        expires_at: item.expires_at.to_string(),
        tenant_id: text::truncate(&item.tenant_id, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_mapping_views(items: &[TenantMapping]) -> Vec<TenantMappingView> {
    let mut views: Vec<TenantMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_mapping_view(item));
    }
    views
}

pub fn sum_tenant_mapping_view(view: &TenantMappingView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_ends_at as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_mapping("code-773", "label-440");
        let view = to_tenant_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
