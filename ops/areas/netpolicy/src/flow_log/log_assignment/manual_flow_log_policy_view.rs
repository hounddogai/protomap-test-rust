use super::*;

/// How screens and exports show a manual flow log policy.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogPolicyView {
    pub code: String,
    pub label: String,
    pub port: String,
    pub ttl_seconds: String,
    pub revision: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_manual_flow_log_policy_view(item: &ManualFlowLogPolicy) -> ManualFlowLogPolicyView {
    ManualFlowLogPolicyView {
        code: item.code.clone(),
        label: item.label.clone(),
        port: item.port.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        revision: item.revision.to_string(),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_flow_log_policy_views(items: &[ManualFlowLogPolicy]) -> Vec<ManualFlowLogPolicyView> {
    let mut views: Vec<ManualFlowLogPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_flow_log_policy_view(item));
    }
    views
}

pub fn sum_manual_flow_log_policy_view(view: &ManualFlowLogPolicyView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_port as f64 + (parsed_ttl_seconds as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_flow_log_policy("code-743", "label-879");
        let view = to_manual_flow_log_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_flow_log_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
