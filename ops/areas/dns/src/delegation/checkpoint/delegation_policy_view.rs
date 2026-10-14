use super::*;

/// How screens and exports show a delegation policy.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPolicyView {
    pub code: String,
    pub title_text: String,
    pub weight: String,
    pub revision_text: String,
    pub audience_text: String,
    pub burst_text: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_delegation_policy_view(item: &DelegationPolicy) -> DelegationPolicyView {
    DelegationPolicyView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        weight: format!("{:.2}", item.weight),
        revision_text: item.revision.to_string(),
        audience_text: item.audience.clone(),
        burst_text: item.burst.to_string(),
        notes: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_policy_views(items: &[DelegationPolicy]) -> Vec<DelegationPolicyView> {
    let mut views: Vec<DelegationPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_policy_view(item));
    }
    views
}

pub fn sum_delegation_policy_view(view: &DelegationPolicyView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_weight + (parsed_revision as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_policy("code-533", "title-832");
        let view = to_delegation_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
