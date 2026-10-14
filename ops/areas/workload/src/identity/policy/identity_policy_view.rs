use super::*;

use acme_kit::text;

/// How screens and exports show a identity policy.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityPolicyView {
    pub code_text: String,
    pub name: String,
    pub floor_value_text: String,
    pub rotation_days: String,
    pub host_text: String,
    pub status_label: String,
}

pub fn to_identity_policy_view(item: &IdentityPolicy) -> IdentityPolicyView {
    IdentityPolicyView {
        code_text: text::truncate(&item.code, 80),
        name: item.name.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        rotation_days: item.rotation_days.to_string(),
        host_text: text::truncate(&item.host, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_policy_views(items: &[IdentityPolicy]) -> Vec<IdentityPolicyView> {
    let mut views: Vec<IdentityPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_policy_view(item));
    }
    views
}

pub fn sum_identity_policy_view(view: &IdentityPolicyView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_floor_value + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_policy("code-700", "name-425");
        let view = to_identity_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
