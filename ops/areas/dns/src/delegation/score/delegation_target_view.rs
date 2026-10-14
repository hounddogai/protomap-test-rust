use super::*;

/// How screens and exports show a delegation target.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTargetView {
    pub code: String,
    pub name: String,
    pub floor_value: String,
    pub ratio_text: String,
    pub active_text: String,
    pub revision_text: String,
    pub port_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_delegation_target_view(item: &DelegationTarget) -> DelegationTargetView {
    DelegationTargetView {
        code: item.code.clone(),
        name: item.name.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        ratio_text: format!("{:.2}", item.ratio),
        active_text: format!("{}", item.active),
        revision_text: item.revision.to_string(),
        port_text: item.port.to_string(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_target_views(items: &[DelegationTarget]) -> Vec<DelegationTargetView> {
    let mut views: Vec<DelegationTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_target_view(item));
    }
    views
}

pub fn sum_delegation_target_view(view: &DelegationTargetView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_floor_value + parsed_ratio + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_target("code-416", "name-171");
        let view = to_delegation_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
