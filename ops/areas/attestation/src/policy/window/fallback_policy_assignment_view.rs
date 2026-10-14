use super::*;

use acme_kit::text;

/// How screens and exports show a fallback policy assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPolicyAssignmentView {
    pub code_text: String,
    pub label_text: String,
    pub key_id_text: String,
    pub phone_text: String,
    pub cluster_text: String,
    pub source_text: String,
    pub floor_value: String,
    pub ratio: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_fallback_policy_assignment_view(item: &FallbackPolicyAssignment) -> FallbackPolicyAssignmentView {
    FallbackPolicyAssignmentView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        key_id_text: item.key_id.clone(),
        phone_text: item.phone.clone(),
        cluster_text: item.cluster.clone(),
        source_text: text::truncate(&item.source, 40),
        floor_value: format!("{:.2}", item.floor_value),
        ratio: format!("{:.2}", item.ratio),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_policy_assignment_views(items: &[FallbackPolicyAssignment]) -> Vec<FallbackPolicyAssignmentView> {
    let mut views: Vec<FallbackPolicyAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_policy_assignment_view(item));
    }
    views
}

pub fn sum_fallback_policy_assignment_view(view: &FallbackPolicyAssignmentView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_floor_value + parsed_ratio + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_policy_assignment("code-490", "label-178");
        let view = to_fallback_policy_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_policy_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
