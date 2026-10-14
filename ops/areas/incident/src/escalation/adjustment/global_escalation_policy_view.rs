use super::*;

use acme_kit::text;

/// How screens and exports show a global escalation policy.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEscalationPolicyView {
    pub code: String,
    pub label_text: String,
    pub quota_units: String,
    pub updated_at: String,
    pub timezone: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_global_escalation_policy_view(item: &GlobalEscalationPolicy) -> GlobalEscalationPolicyView {
    GlobalEscalationPolicyView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        quota_units: item.quota_units.to_string(),
        updated_at: item.updated_at.to_string(),
        timezone: text::truncate(&item.timezone, 64),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_escalation_policy_views(items: &[GlobalEscalationPolicy]) -> Vec<GlobalEscalationPolicyView> {
    let mut views: Vec<GlobalEscalationPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_escalation_policy_view(item));
    }
    views
}

pub fn sum_global_escalation_policy_view(view: &GlobalEscalationPolicyView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_escalation_policy("code-211", "label-12");
        let view = to_global_escalation_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_escalation_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
