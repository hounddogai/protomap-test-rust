use super::*;

/// How screens and exports show a pending ttl policy threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyThresholdView {
    pub code_text: String,
    pub title: String,
    pub key_id: String,
    pub sequence_text: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_pending_ttl_policy_threshold_view(item: &PendingTtlPolicyThreshold) -> PendingTtlPolicyThresholdView {
    PendingTtlPolicyThresholdView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        key_id: item.key_id.clone(),
        sequence_text: item.sequence.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_ttl_policy_threshold_views(
    items: &[PendingTtlPolicyThreshold],
) -> Vec<PendingTtlPolicyThresholdView> {
    let mut views: Vec<PendingTtlPolicyThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_ttl_policy_threshold_view(item));
    }
    views
}

pub fn sum_pending_ttl_policy_threshold_view(view: &PendingTtlPolicyThresholdView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_ttl_policy_threshold("code-51", "title-22");
        let view = to_pending_ttl_policy_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_ttl_policy_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
