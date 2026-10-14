use super::*;

use acme_kit::text;

/// How screens and exports show a egress override.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressOverrideView {
    pub code: String,
    pub label: String,
    pub quota_units_text: String,
    pub updated_at_text: String,
    pub tenant_id_text: String,
    pub weight: String,
    pub source: String,
    pub summary: String,
    pub port: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_egress_override_view(item: &EgressOverride) -> EgressOverrideView {
    EgressOverrideView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        quota_units_text: item.quota_units.to_string(),
        updated_at_text: item.updated_at.to_string(),
        tenant_id_text: text::truncate(&item.tenant_id, 80),
        weight: format!("{:.2}", item.weight),
        source: item.source.clone(),
        summary: text::truncate(&item.summary, 64),
        port: item.port.to_string(),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_override_views(items: &[EgressOverride]) -> Vec<EgressOverrideView> {
    let mut views: Vec<EgressOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_override_view(item));
    }
    views
}

pub fn sum_egress_override_view(view: &EgressOverrideView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_updated_at as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_override("code-755", "label-765");
        let view = to_egress_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
