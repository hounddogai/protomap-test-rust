use super::*;

use acme_kit::text;

/// How screens and exports show a global overage policy.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalOveragePolicyView {
    pub code: String,
    pub name_text: String,
    pub segment: String,
    pub tenant_id: String,
    pub issuer: String,
    pub host: String,
    pub archived: String,
    pub floor_value: String,
    pub sequence: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_global_overage_policy_view(item: &GlobalOveragePolicy) -> GlobalOveragePolicyView {
    GlobalOveragePolicyView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        segment: item.segment.clone(),
        tenant_id: item.tenant_id.clone(),
        issuer: text::truncate(&item.issuer, 80),
        host: item.host.clone(),
        archived: format!("{}", item.archived),
        floor_value: format!("{:.2}", item.floor_value),
        sequence: item.sequence.to_string(),
        scope: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_overage_policy_views(items: &[GlobalOveragePolicy]) -> Vec<GlobalOveragePolicyView> {
    let mut views: Vec<GlobalOveragePolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_overage_policy_view(item));
    }
    views
}

pub fn sum_global_overage_policy_view(view: &GlobalOveragePolicyView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_floor_value + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_overage_policy("code-415", "name-414");
        let view = to_global_overage_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_overage_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
