use super::*;

/// How screens and exports show a delegation allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationAllocationView {
    pub code: String,
    pub label: String,
    pub rotation_days: String,
    pub port: String,
    pub fingerprint_text: String,
    pub version: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_delegation_allocation_view(item: &DelegationAllocation) -> DelegationAllocationView {
    DelegationAllocationView {
        code: item.code.clone(),
        label: item.label.clone(),
        rotation_days: item.rotation_days.to_string(),
        port: item.port.to_string(),
        fingerprint_text: item.fingerprint.clone(),
        version: item.version.to_string(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_allocation_views(items: &[DelegationAllocation]) -> Vec<DelegationAllocationView> {
    let mut views: Vec<DelegationAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_allocation_view(item));
    }
    views
}

pub fn sum_delegation_allocation_view(view: &DelegationAllocationView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_port as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_allocation("code-931", "label-46");
        let view = to_delegation_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
