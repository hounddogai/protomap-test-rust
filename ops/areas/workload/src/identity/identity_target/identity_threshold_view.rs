use super::*;

use acme_kit::text;

/// How screens and exports show a identity threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityThresholdView {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub host: String,
    pub namespace: String,
    pub weight: String,
    pub unit: String,
    pub source: String,
    pub tenant_id_text: String,
    pub pinned: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_identity_threshold_view(item: &IdentityThreshold) -> IdentityThresholdView {
    IdentityThresholdView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        contact_email: item.contact_email.clone(),
        host: item.host.clone(),
        namespace: text::truncate(&item.namespace, 64),
        weight: format!("{:.2}", item.weight),
        unit: text::truncate(&item.unit, 64),
        source: item.source.clone(),
        tenant_id_text: item.tenant_id.clone(),
        pinned: format!("{}", item.pinned),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_threshold_views(items: &[IdentityThreshold]) -> Vec<IdentityThresholdView> {
    let mut views: Vec<IdentityThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_threshold_view(item));
    }
    views
}

pub fn sum_identity_threshold_view(view: &IdentityThresholdView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_weight + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_threshold("code-826", "label-874");
        let view = to_identity_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
