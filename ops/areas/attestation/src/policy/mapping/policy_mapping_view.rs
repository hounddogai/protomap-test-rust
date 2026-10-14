use super::*;

use acme_kit::text;

/// How screens and exports show a policy mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMappingView {
    pub code_text: String,
    pub label_text: String,
    pub tenant_id: String,
    pub updated_at: String,
    pub archived: String,
    pub phone: String,
    pub quantity: String,
    pub segment: String,
    pub cost: String,
    pub ttl_seconds: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_policy_mapping_view(item: &PolicyMapping) -> PolicyMappingView {
    PolicyMappingView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        tenant_id: item.tenant_id.clone(),
        updated_at: item.updated_at.to_string(),
        archived: format!("{}", item.archived),
        phone: item.phone.clone(),
        quantity: item.quantity.to_string(),
        segment: item.segment.clone(),
        cost: format!("{:.2}", item.cost),
        ttl_seconds: item.ttl_seconds.to_string(),
        owner_id: text::truncate(&item.owner_id, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_mapping_views(items: &[PolicyMapping]) -> Vec<PolicyMappingView> {
    let mut views: Vec<PolicyMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_mapping_view(item));
    }
    views
}

pub fn sum_policy_mapping_view(view: &PolicyMappingView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_quantity as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_mapping("code-857", "label-870");
        let view = to_policy_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
