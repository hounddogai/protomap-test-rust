use super::*;

/// How screens and exports show a escalation policy.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPolicyView {
    pub code_text: String,
    pub name: String,
    pub attempts_text: String,
    pub tenant_id_text: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_escalation_policy_view(item: &EscalationPolicy) -> EscalationPolicyView {
    EscalationPolicyView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        attempts_text: item.attempts.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_policy_views(items: &[EscalationPolicy]) -> Vec<EscalationPolicyView> {
    let mut views: Vec<EscalationPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_policy_view(item));
    }
    views
}

pub fn sum_escalation_policy_view(view: &EscalationPolicyView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_escalation_policy("code-888", "name-813");
        let view = to_escalation_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_escalation_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
