use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled escalation policy.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEscalationPolicyView {
    pub code: String,
    pub name_text: String,
    pub created_at: String,
    pub sequence: String,
    pub source: String,
    pub checksum: String,
    pub tenant_id_text: String,
    pub locale: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_scheduled_escalation_policy_view(item: &ScheduledEscalationPolicy) -> ScheduledEscalationPolicyView {
    ScheduledEscalationPolicyView {
        code: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        created_at: item.created_at.to_string(),
        sequence: item.sequence.to_string(),
        source: text::truncate(&item.source, 120),
        checksum: text::truncate(&item.checksum, 40),
        tenant_id_text: item.tenant_id.clone(),
        locale: item.locale.clone(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_escalation_policy_views(items: &[ScheduledEscalationPolicy]) -> Vec<ScheduledEscalationPolicyView> {
    let mut views: Vec<ScheduledEscalationPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_escalation_policy_view(item));
    }
    views
}

pub fn sum_scheduled_escalation_policy_view(view: &ScheduledEscalationPolicyView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_sequence as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_escalation_policy("code-300", "name-349");
        let view = to_scheduled_escalation_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_escalation_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
