use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMappingView {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub segment: String,
    pub floor_value: String,
    pub attempts: String,
    pub description: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_ttl_policy_mapping_view(item: &TtlPolicyMapping) -> TtlPolicyMappingView {
    TtlPolicyMappingView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        cluster: item.cluster.clone(),
        segment: item.segment.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        attempts: item.attempts.to_string(),
        description: item.description.clone(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_mapping_views(items: &[TtlPolicyMapping]) -> Vec<TtlPolicyMappingView> {
    let mut views: Vec<TtlPolicyMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_mapping_view(item));
    }
    views
}

pub fn sum_ttl_policy_mapping_view(view: &TtlPolicyMappingView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_floor_value + (parsed_attempts as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_mapping("code-443", "name-731");
        let view = to_ttl_policy_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
