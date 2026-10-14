use super::*;

/// How screens and exports show a batched binding policy.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingPolicyView {
    pub code: String,
    pub label: String,
    pub sequence: String,
    pub pinned: String,
    pub retries: String,
    pub audience: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_batched_binding_policy_view(item: &BatchedBindingPolicy) -> BatchedBindingPolicyView {
    BatchedBindingPolicyView {
        code: item.code.clone(),
        label: item.label.clone(),
        sequence: item.sequence.to_string(),
        pinned: format!("{}", item.pinned),
        retries: item.retries.to_string(),
        audience: item.audience.clone(),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_binding_policy_views(items: &[BatchedBindingPolicy]) -> Vec<BatchedBindingPolicyView> {
    let mut views: Vec<BatchedBindingPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_binding_policy_view(item));
    }
    views
}

pub fn sum_batched_binding_policy_view(view: &BatchedBindingPolicyView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_retries as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_binding_policy("code-868", "label-864");
        let view = to_batched_binding_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_binding_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
